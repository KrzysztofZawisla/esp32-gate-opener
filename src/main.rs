use core::borrow::Borrow;
use std::sync::OnceLock;

use anyhow::Result;
use embassy_futures::join::join;
use embassy_time::Duration as TimeDuration;
use embassy_time::{Instant, Timer};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfiguration};
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::gpio::{ADCPin, AnyIOPin, AnyOutputPin, PinDriver, Pull};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::block_on;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp::EspSntp;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log::{info, warn};

mod config;
mod config_storage;
mod gate;
mod homeassistant;
mod http;
mod ota;
mod pure;
mod state;

use config::*;
use pure::Command;
use state::{battery_pct, refresh_status, take_command};

// Keep the SNTP client alive for the whole run; dropping it would stop time
// synchronization (and free the underlying task).
static SNTP: OnceLock<EspSntp<'static>> = OnceLock::new();

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    // With CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE the running slot must confirm
    // itself as valid; otherwise the bootloader reverts on the *next* reboot
    // (even after a normal power cycle), not just after a crash. Confirm as soon
    // as the firmware is past basic init — a crash before this point still rolls back.
    if let Ok(mut ota) = esp_idf_svc::ota::EspOta::new() {
        if let Err(error) = ota.mark_running_slot_valid() {
            warn!("Failed to mark OTA slot valid: {error}");
        }
    }

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs_partition = EspDefaultNvsPartition::take()?;
    config_storage::init(nvs_partition.clone());

    let mut pins = gate::GatePins {
        open_relay: PinDriver::output(AnyOutputPin::from(peripherals.pins.gpio4))?,
        close_relay: PinDriver::output(AnyOutputPin::from(peripherals.pins.gpio23))?,
        lamp_green: PinDriver::output(AnyOutputPin::from(peripherals.pins.gpio27))?,
        lamp_red: PinDriver::output(AnyOutputPin::from(peripherals.pins.gpio14))?,
        open_sensor: PinDriver::input(AnyIOPin::from(peripherals.pins.gpio25))?,
        closed_sensor: PinDriver::input(AnyIOPin::from(peripherals.pins.gpio26))?,
        obstacle_sensor: PinDriver::input(AnyIOPin::from(peripherals.pins.gpio33))?,
        obstacle_active_level: env!("OBSTACLE_ACTIVE_LEVEL") == "high",
    };
    pins.open_sensor.set_pull(Pull::Up)?;
    pins.closed_sensor.set_pull(Pull::Up)?;
    pins.obstacle_sensor.set_pull(Pull::Up)?;

    let adc = AdcDriver::new(peripherals.adc1)?;
    let adc_config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let mut battery_channel = AdcChannelDriver::new(&adc, peripherals.pins.gpio36, &adc_config)?;

    pins.open_relay.set_low()?;
    pins.close_relay.set_low()?;
    gate::set_lamp(&mut pins, false, false)?;

    refresh_status(&pins.open_sensor, &pins.closed_sensor);

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs_partition))?,
        sys_loop,
    )?;
    wifi.set_configuration(&WifiConfiguration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    }))?;
    wifi.start()?;
    for attempt in 0..6 {
        match wifi.connect().and_then(|_| wifi.wait_netif_up()) {
            Ok(()) => {
                info!("WiFi connected");
                if let Ok(ip_info) = wifi.wifi().sta_netif().get_ip_info() {
                    info!("IP address: {:?}", ip_info.ip);
                }
                break;
            }
            Err(error) => {
                warn!("WiFi connect failed (attempt {attempt}/6): {error}");
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    }

    homeassistant::connect_mqtt();

    // Best-effort time synchronization over the (now connected) network so
    // logs and any future time-stamped features agree with real time. SNTP
    // queries the pool in the background until it gets a sync; failures here
    // are non-fatal because the device keeps working without a clock.
    match EspSntp::new_with_callback(&Default::default(), |offset| {
        info!(
            "Clock synchronized via NTP (offset {} ms)",
            offset.as_millis()
        );
    }) {
        Ok(sntp) => {
            let _ = SNTP.set(sntp);
            info!("SNTP time synchronization enabled");
        }
        Err(error) => warn!("Failed to enable SNTP: {error}"),
    }

    let server = http::start_http_server()?;

    info!("Running 24/7 — WiFi, MQTT and HTTP are always on");
    block_on(join(
        gate_task(&mut pins),
        telemetry_task(&mut wifi, &mut battery_channel),
    ));

    drop(server);
    Ok(())
}

async fn gate_task(pins: &mut gate::GatePins) {
    loop {
        refresh_status(&pins.open_sensor, &pins.closed_sensor);
        let command = take_command();
        if command != Command::None {
            if battery_pct() < config_storage::battery_min_pct() {
                warn!(
                    "Battery too low ({}%), refusing to move the gate",
                    battery_pct()
                );
                state::set_fault(pure::Fault::BATTERY);
                homeassistant::publish_fault();
                continue;
            }
            state::clear_fault(pure::Fault::BATTERY);
            let mut current = command;
            loop {
                current = gate::handle_command(current, pins)
                    .await
                    .unwrap_or(Command::None);
                refresh_status(&pins.open_sensor, &pins.closed_sensor);
                homeassistant::publish_obstacle();
                if current == Command::None {
                    homeassistant::publish_status();
                    homeassistant::publish_fault();
                    break;
                }
            }
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
    }
}

async fn telemetry_task<AdcPin, AdcModule>(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    battery_channel: &mut AdcChannelDriver<'static, AdcPin, AdcModule>,
) where
    AdcPin: ADCPin,
    AdcModule: Borrow<AdcDriver<'static, AdcPin::Adc>>,
{
    const RECONNECT_INTERVAL_S: u64 = 5;
    let mut last_periodic = Instant::now();
    let mut last_reconnect = Instant::now();
    loop {
        match wifi.is_connected() {
            Ok(true) => {}
            _ => {
                if Instant::now().saturating_duration_since(last_reconnect)
                    >= TimeDuration::from_secs(RECONNECT_INTERVAL_S)
                {
                    last_reconnect = Instant::now();
                    warn!("WiFi connection lost, reconnecting");
                    let _ = wifi.disconnect();
                    let _ = wifi.connect();
                    let _ = wifi.wait_netif_up();
                }
            }
        }

        if state::mqtt_connected() {
            let interval = TimeDuration::from_secs(config_storage::telemetry_interval_s());
            let periodic_due = Instant::now().saturating_duration_since(last_periodic) >= interval;
            if periodic_due {
                last_periodic = Instant::now();
            }
            if state::take_battery_publish_request() || periodic_due {
                homeassistant::publish_battery(battery_channel);
            }
        }

        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
    }
}
