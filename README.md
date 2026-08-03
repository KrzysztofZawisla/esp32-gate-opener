# ESP32 Gate Opener

Solar-powered gate opener for a driveway gate, built on an ESP32 with Rust (ESP-IDF).
The device runs **24/7** — WiFi, MQTT and HTTP are always on, so commands are executed
instantly and the gate state is continuously reported to Home Assistant. An MQTT command
pulses the matching relay (`OPEN` / `CLOSE`), which presses the gate controller's wired
button input. A single **two-color lamp** signals the gate state (green = opening,
red = closing). The firmware uses **async/await (Embassy-style) on top of ESP-IDF**,
so all subsystems run concurrently without thread stacks.

Features:
- **OTA updates** over HTTP (`POST /ota`) — dual OTA slots + bootloader-level rollback.
- **Runtime config in NVS** with compile-time fallbacks (`GET/POST /config`) — change the
  API key, pulse length, timeouts etc. without reflashing.
- **MQTT authentication** (username/password) and an optional `X-Api-Key` header on the HTTP endpoints.
- **Fail-open safety**: the gate is always left open or closed, never stopped mid-travel.
- **Battery lockout**: below `BATTERY_MIN_PCT` the gate refuses to move (fault reported).
- **Sensor fault detection**: both reed switches reading "closed" at once → `error` state.
- **Home Assistant MQTT discovery** — cover, battery, voltage, obstruction and fault entities
  appear automatically.

---

## How it works

The firmware is structured around a single async executor driven by
`esp_idf_hal::task::block_on` and a real time driver provided by `esp-idf-svc`'s
`embassy-time-driver` feature. Two async tasks run concurrently via
`embassy_futures::join`:

| Task | Responsibility |
|---|---|
| **Gate task** | Polls the command topic / HTTP `/open` `/close` every `SENSOR_POLL_MS`. On a command it pulses the relay for `GATE_PULSE_MS`, keeps the lamp on and monitors the position + obstacle sensors until the end position or `MOTION_TIMEOUT_S`, then publishes the new state. |
| **Telemetry task** | Every `TELEMETRY_INTERVAL_S`: checks the WiFi link (reconnects if lost), and publishes `online`, gate state, obstacle state and battery level. |

The WiFi, MQTT and HTTP server run in parallel:

1. **WiFi** connects at boot (with retries) and stays up; the telemetry task reconnects
   if the link drops. MQTT auto-reconnects via the broker client.
2. **MQTT** subscribes to `gate/command`. A retained `open` / `close` message (or a live
   publish, since the device is always awake) is executed immediately. MQTT discovery
   configs are (re)published on every (re)connect so Home Assistant entities stay valid.
3. **HTTP** (`POST /open`, `/close`, `/config`, `/ota`; `GET /status`, `/config`) is
   always reachable at the device IP, even if WiFi is still reconnecting.
4. **Obstacle safety (fail-open)**: if the through-beam photocell reports a blocked
   driveway, the device refuses to start closing; if the beam is broken **during**
   closing, the gate is reversed back to fully open. Timeouts never leave the gate in
   an undefined state: if the gate does not reach the closed limit in time it is
   reversed to open, and if opening is not confirmed the open pulse is retried.
   The gate is always left either **open** or **closed** — never stopped midway.
5. **Battery lockout**: if the last measured battery level drops below `BATTERY_MIN_PCT`,
   motion commands are refused (the gate stays where it is) and a fault is reported.
6. **Fault detection**: if both reed switches read "closed" simultaneously — a
   physically impossible state — the device enters the `error` status and publishes a
   fault. A brief `GRACE_MS` delay after each relay pulse lets the gate controller and
   mechanics settle before the sensors are polled.

Because the device is always on, the power budget is dominated by the always-on WiFi/MQTT
link (modem-sleep when idle). See [Power budget and battery sizing](#power-budget-and-battery-sizing).

### Gate motion sequences

**Opening** (`open` command):
1. If the open sensor already reads "open", the command is a no-op.
2. Status → `opening`, lamp → green, relay pulse for `GATE_PULSE_MS` (1 s by default).
3. `GRACE_MS` settle delay, then poll the open sensor up to `MOTION_TIMEOUT_S`.
4. If the open limit was not reached in time, the pulse is retried **once** (pulse +
   settle + wait again).
5. Status → `open`, lamp off.

**Closing** (`close` command):
1. If the closed sensor already reads "closed", the command is a no-op.
2. If the obstacle beam is broken → **refuse to close**, status stays `open`,
   obstacle → `on`, no motion.
3. Status → `closing`, lamp → red, relay pulse for `GATE_PULSE_MS`, `GRACE_MS` settle.
4. Poll the closed sensor and the obstacle beam every `SENSOR_POLL_MS` up to
   `MOTION_TIMEOUT_S`:
   - Beam broken while closing → **reverse to open** (open pulse + wait for the open
     sensor), obstacle → `on`.
   - Closed sensor reached → status → `closed`.
   - Timeout → **fail-open**: reverse to fully open (never left stopped mid-way).

**Interrupts**: a new `open`/`close` command arriving while a motion is in progress
**aborts the current action** and starts the new one (the command slot is single-valued,
last command wins). E.g. `close` followed quickly by `open` stops the closing pulse and
opens the gate instead.

### Boot sequence

1. ESP-IDF link patches + logger; **the running OTA slot is confirmed valid**
   (see [OTA and rollback](#ota-and-rollback)).
2. NVS is opened and the runtime configuration is loaded (falling back to the
   compile-time defaults).
3. GPIOs and the battery ADC are initialized; relays and lamp start off; the initial
   gate status is read from the sensors.
4. WiFi connects: up to **6 attempts, 5 s apart**. If it still fails, the firmware
   **continues anyway** — the HTTP server still runs and MQTT keeps trying to connect.
5. MQTT client starts (with automatic reconnection) and the HTTP server binds port 80.
6. The gate and telemetry tasks run forever.

> The battery percentage starts at **100 %** and is only refreshed by the first
> telemetry sample (every `TELEMETRY_INTERVAL_S`). The battery lockout therefore has
> no effect for the first ~60 s after boot. On a bad ADC read the battery is reported
> as **0 %** and the voltage topic is not published.

---

## Parts list (BOM)

Everything needed to build one unit (quantities and typical sources for ~2026).

### Electronics

| # | Part | Qty | Typical source / link | Notes |
|---|------|-----|-----------------------|-------|
| 1 | ESP32 DevKit (38-pin), e.g. ESP32-WROOM-32 / NodeMCU-32S | 1 | AliExpress / local shop / [Mouser](https://www.mouser.com/) | Used GPIOs: 4, 14, 23, 25, 26, 27, 33, 36 |
| 2 | 2-channel relay module (optocoupler, 5 V) | 1 | e.g. classic blue relay module | One channel per direction (`OPEN`, `CLOSE`) |
| 3 | Solar panel 12 V | 1 | 20–30 W recommended (see power budget) | Poly/mono; sized for winter |
| 4 | Solar charge controller (PWM, 12 V, with low-voltage disconnect) | 1 | e.g. generic 10 A PWM controller | Protect battery from deep discharge |
| 5 | Battery 12 V | 1 | 12 V 7–12 Ah gel/AGM | See power budget |
| 6 | Buck converter 12 V → 5 V (MP1584 / XL4015 / AMS1117) | 1 | AliExpress / LCSC | Powers ESP32 + relay module |
| 7 | Two-color lamp (common cathode) or bicolor LED | 1 | LED module / lamp shop | Green + red; see pin map |
| 8 | Through-beam photocell (IR transmitter + receiver pair) | 1 | gate/industrial sensor shop | Beam across driveway at 30–50 cm |
| 9 | Reed switches or limit switches | 2 | local electronics shop | Open + closed position |
| 10 | N-MOSFETs (e.g. IRLZ44N) for 12 V lamp module | 2 | local shop | Only if driving a 12 V lamp module |
| 11 | Resistor kit (incl. 100 kΩ, 20 kΩ, 100 Ω) | 1 | any kit | Voltage divider + LED/relay current limits |
| 12 | Prototype PCB / perfboard + headers | 1 | local shop | Optional, for mounting |
| 13 | Enclosure (IP65 weatherproof, ~150×100×60 mm) | 1 | electrical shop / 3D print | Outdoor gate location |
| 14 | Fuse (2–5 A) + wiring + terminal blocks | 1 set | local shop | Between battery and loads |

### Wiring / consumables

| # | Part | Qty | Notes |
|---|------|-----|-------|
| 15 | Silicone wire (0.5–1.5 mm²) | ~5 m | Power + signals |
| 16 | Heat-shrink tubing | 1 set | Insulation |
| 17 | Cable glands / grommets | 2–4 | Into the enclosure |
| 18 | Mounting screws / brackets | 1 set | Panel + enclosure |

### Optional

| # | Part | Qty | Notes |
|---|------|-----|-------|
| 19 | Wemos/ESP32 USB power adapter (5 V 2 A) | 1 | Bench testing without solar |
| 20 | JST-XH or screw connectors | 1 set | Releasable battery/power joints |
| 21 | TVS / zener diode on the ADC input | 1 | Extra protection of GPIO36 |

> Quantities assume a single gate installation. Prices are not listed because they vary
> heavily by market and time; expect the electronics (items 1–12) in the ~15–30 € / 20–40 $
> range from AliExpress, or roughly double bought locally.

---

## Hardware requirements

### Core components

| Component | Recommendation | Notes |
|---|---|---|
| ESP32 DevKit | ESP32-WROOM-32 / NodeMCU-32S (38 pins) | GPIO4, 14, 23, 25, 26, 27, 33, 36 are used |
| Relay module | 2-channel relay module with optocoupler (e.g. classic blue 5 V module) | One channel for `OPEN`, one for `CLOSE`; galvanic isolation |
| Solar panel | **20–30 W**, 12 V | Sized by the 24/7 power budget below |
| Battery | **12 V gel/AGM, 7–12 Ah** | See power budget; must survive nights |
| Charge controller | PWM solar controller (12 V) with low-voltage disconnect | Matches battery chemistry |
| Voltage regulator | Buck converter 12 V → 5 V (MP1584/XL4015) | Feeds the ESP32 and relay |
| Two-color lamp | Bicolor LED / lamp module (common cathode), green + red | Single lamp; green on GPIO27, red on GPIO14 |
| Battery voltage divider | 2 resistors, e.g. 100 kΩ + 20 kΩ | Scales battery voltage to the ADC input range |
| Open position sensor | Reed switch or limit switch | Magnet at the fully-open position |
| Closed position sensor | Reed switch or limit switch | Magnet at the fully-closed position |
| Obstacle sensor | Through-beam photocell (transmitter + receiver) | Beam across the driveway; broken beam = something in the gate path |
| Gate controller | Drive controller with wired button / receiver input | Usually terminals `OPEN`, `CLOSE`, `STOP`, `COM` |

### Why a relay?

GPIO4 outputs only 3.3 V with a few mA — it cannot drive a relay coil or the gate
controller input directly. The relay:
- switches the gate controller circuit powered from its own supply (e.g. 12 V / 24 V),
- galvanically isolates the ESP32 electronics from the gate's power section.

### Pin map

| Function | GPIO | Direction | Notes |
|---|---|---|---|
| Open relay | GPIO4 | Output | High for `GATE_PULSE_MS`; drives `OPEN` input of the gate controller |
| Close relay | GPIO23 | Output | High for `GATE_PULSE_MS`; drives `CLOSE` input of the gate controller |
| Lamp – green | GPIO27 | Output | Active high; via MOSFET/LED driver |
| Lamp – red | GPIO14 | Output | Active high; via MOSFET/LED driver |
| Open sensor | GPIO25 | Input, pull-up | Reed switch to GND; low = gate fully open |
| Closed sensor | GPIO26 | Input, pull-up | Reed switch to GND; low = gate fully closed |
| Obstacle sensor | GPIO33 | Input, pull-up | Through-beam receiver output; see `OBSTACLE_ACTIVE_LEVEL` |
| Battery ADC | GPIO36 (VP) | Input (ADC1) | Reads the voltage divider; never drive it as output |

The two lamp pins drive **one** physical two-color lamp; the firmware never lights
both colors at the same time (green = opening, red = closing, off = stopped).
Avoid strapping pins (GPIO0, GPIO2, GPIO12, GPIO15) — the pins above are safe.

---

## Wiring

### Power section

```
                        ┌──────────────────────────────────────┐
    Solar panel         │  Charge controller                   │
       12V ─────────────▶  (PWM, battery type)                 │
                        └──────────────────────┬───────────────┘
                                               │
                                       12V battery (gel/AGM)
                                      ┌─────────┴────────┐
                                      │                  │
                               Buck 12V→5V         Gate controller supply
                                      │                  │
                                      ▼                  │
                                   +5V ──▶ ESP32 VIN + relay module VCC
                                      GND shared
```

### Battery voltage divider

```
    Battery +12V ──┬── R1 (100 kΩ) ──┬── GPIO36 (ADC1)
                   │                  │
                   │              C1 (100 nF, optional)
                   │                  │
    Battery GND ───┴──────────────────┴── GND
```

With R1 = 100 kΩ and R2 = 20 kΩ, the divider ratio is `(R1+R2)/R2 = 6`. At 12.6 V
the pin sees 2.1 V — safely inside the ADC input range. Set `BATTERY_DIVIDER_RATIO`
to your actual ratio. Calibrate `BATTERY_FULL_MV` / `BATTERY_EMPTY_MV` to your
battery chemistry (defaults: 12600 / 11500 mV for 12 V lead-acid).

### Control section

```
    ESP32 DevKit               Relay module (2ch)            Gate controller
    ┌────────────┐              ┌──────────────┐           ┌───────────────┐
    │  GPIO4  ───┼─────────────▶│ IN1          │           │               │
    │ GPIO23  ───┼─────────────▶│ IN2          │           │               │
    │  VIN   ────┼─────────────▶│ VCC (5V)     │           │               │
    │  GND   ────┼─────────────▶│ GND          │           │               │
    └────────────┘              │ NO1 ─────────┼──────────▶│ OPEN          │
                                │ NO2 ─────────┼──────────▶│ CLOSE         │
                                │ COM1 ────────┼──────────▶│ COM           │
                                │ COM2 ────────┼──────────▶│ COM           │
                                └──────────────┘           └───────────────┘

    Through-beam obstacle sensor (across the driveway):
      Transmitter (+5V, GND) ── on one gate post
      Receiver   OUT ───────▶ GPIO33      (pull-up in firmware)
      Receiver   VCC/GND ────▶ +5V / GND  (on the other post)

    Two-color lamp (common cathode, one housing):
      GPIO27 ──▶ gate resistor (100 Ω) ──▶ green anode
      GPIO14 ──▶ gate resistor (100 Ω) ──▶ red anode
      (or drive a 12 V lamp module via two N-MOSFETs from GPIO27/GPIO14)
```

1. **Power**: solar panel → charge controller → battery → buck → `VIN` of the ESP32
   and `VCC` of the relay module; common `GND`.
2. **Relays**: `GPIO4` → `IN1` (`OPEN` input) and `GPIO23` → `IN2` (`CLOSE` input).
   The relay `NO`–`COM` contacts connect in parallel with the gate controller's
   `OPEN`–`COM` and `CLOSE`–`COM` terminals (where the wired buttons or receiver
   outputs normally connect).
3. **Position sensors**: reed switch (NO) between `GPIO25` and `GND` mounted so the
   magnet closes it when the gate is fully open; same for `GPIO26` (fully closed).
4. **Obstacle sensor**: through-beam photocell with the beam crossing the driveway at
   ~30–50 cm above ground. Receiver output → `GPIO33`. Set `OBSTACLE_ACTIVE_LEVEL` to
   the level the pin reads **when the beam is broken** (`low` for NPN/open-collector
   receivers, `high` for PNP).
5. **Lamp**: one two-color lamp — green anode to `GPIO27`, red anode to `GPIO14`,
   common cathode to GND. For a 12 V lamp module use two N-MOSFETs with gate pull-downs.
6. **Battery voltage**: voltage divider (see above) → `GPIO36`.

> **Active-low relay modules**: the classic blue optocoupler relay modules trigger on
> **LOW** (`LOW` = ON). If yours does that, the HIGH pulse from GPIO4/GPIO23 will not
> switch it. Solutions: use an active-high module, add an inverting transistor between
> the GPIO and `IN`, or invert the pulse logic in `src/main.rs`.

---

## Power budget and battery sizing

Because the device runs 24/7, the **idle WiFi link dominates** the budget. The ESP32
in modem-sleep (WiFi connected, no traffic) draws roughly 30–60 mA at 3.3 V. Measured
ballpark values for a 5 V system:

| State | Current @ 5 V |
|---|---|
| Modem-sleep (WiFi connected, idle) | ~40–70 mA |
| Active MQTT/HTTP traffic | ~80–120 mA |
| Gate motion monitoring (lamp on, WiFi on) | ~120 mA |
| Relay pulse (1 s) | +~70 mA |

Daily energy with a 12 V system:

```
Idle 24h:    50 mA @ 5 V  = 250 mW → 6 Wh/day ≈ 0.5 Ah/day @ 12 V
+ MQTT/HTTP overhead       → ~0.6–0.9 Ah/day @ 12 V
+ a few open/close cycles  → +0.1–0.2 Ah/day
```

**Total ≈ 0.7–1.1 Ah/day at 12 V.** That drives the sizing:
- A **12 V 7 Ah** battery alone lasts ~1 week of darkness.
- To survive **2–3 winter days** without sun, use **12 V 12 Ah**.
- The panel must **recharge ~1 Ah/day** even in winter: use **20–30 W**. In summer
  20 W is generous; in winter 30 W is safer.

---

## Configuration

All settings are compile-time values in the `[env]` section of `.cargo/config.toml`
(read with `env!`). The **HTTP API key, pulse length, timeouts and battery threshold
can be overridden at runtime** and are stored in NVS (`GET/POST /config`) — the NVS
values win over the compile-time defaults.

| Variable | Description | Default | Runtime |
|---|---|---|---|
| `SSID` | Wi-Fi network name | `YOUR_WIFI_SSID` | — |
| `PASSWORD` | Wi-Fi password (WPA2) | `YOUR_WIFI_PASSWORD` | — |
| `MQTT_BROKER` | Broker URL (`mqtt://host:port`) | `mqtt://192.168.1.100:1883` | — |
| `MQTT_USERNAME` | MQTT username; empty = no auth | `` | — |
| `MQTT_PASSWORD` | MQTT password (with `MQTT_USERNAME`) | `` | — |
| `MQTT_COMMAND_TOPIC` | Topic receiving `open` / `close` | `gate/command` | — |
| `MQTT_STATUS_TOPIC` | Topic with `open` / `closed` / `opening` / `closing` / `stopped` / `error` | `gate/status` | — |
| `MQTT_AVAILABILITY_TOPIC` | Topic with `online` / `offline` (LWT) | `gate/availability` | — |
| `MQTT_BATTERY_TOPIC` | Topic with battery level in % | `gate/battery` | — |
| `MQTT_BATTERY_VOLTAGE_TOPIC` | Topic with battery voltage in V | `gate/battery_voltage` | — |
| `MQTT_OBSTACLE_TOPIC` | Topic with obstruction state `on` / `off` | `gate/obstacle` | — |
| `MQTT_FAULT_TOPIC` | Topic with fault state `on` / `off` | `gate/fault` | — |
| `OBSTACLE_ACTIVE_LEVEL` | Level on GPIO33 that means "beam broken" | `low` (NPN) | — |
| `BATTERY_DIVIDER_RATIO` | `(R1+R2)/R2` of the voltage divider | `6.0` | — |
| `BATTERY_FULL_MV` | Battery voltage at 100% | `12600` (12 V lead-acid) | — |
| `BATTERY_EMPTY_MV` | Battery voltage at 0% | `11500` | — |
| `BATTERY_MIN_PCT` | Below this % the gate refuses to move | `20` | ✅ |
| `MOTION_TIMEOUT_S` | Max motion time before giving up on the sensor | `45` | ✅ |
| `TELEMETRY_INTERVAL_S` | How often the telemetry task publishes state/battery | `60` | ✅ |
| `GRACE_MS` | Delay after each relay pulse before polling sensors | `300` | ✅ |
| `GATE_PULSE_MS` | Relay pulse length | `1000` (code constant) | ✅ |
| `HTTP_API_KEY` | Optional `X-Api-Key` header required by `/open` `/close` `/ota` `/config`; empty = disabled | `` | ✅ |
| `ESP_LOG` | Log level | `info` | — |

Code constants in `src/config.rs`: `LISTEN_PORT` (HTTP, 80), `GATE_PULSE_MS` (1000 ms,
runtime-overridable), `SENSOR_POLL_MS` (100 ms), `MQTT_KEEPALIVE_S` (10 s), and the GPIO
pin numbers (in `src/main.rs`).

> Runtime-overridable values are stored in NVS under the `gate` namespace and always
> win over the compile-time defaults. Everything else requires a rebuild + reflash.

### Runtime config (NVS)

`GET /config` returns the effective settings (NVS values, falling back to defaults)
as JSON:

```json
{"http_api_key":"","battery_min_pct":20,"grace_ms":300,"motion_timeout_s":45,"gate_pulse_ms":1000,"telemetry_interval_s":60}
```

`POST /config` (requires the `X-Api-Key` header) updates any subset via query
parameters and persists them to NVS:

```
curl -X POST -H "X-Api-Key: <HTTP_API_KEY>" "http://<device-ip>/config?gate_pulse_ms=700&battery_min_pct=25&grace_ms=500"
```

Supported parameters: `http_api_key`, `battery_min_pct`, `grace_ms`, `motion_timeout_s`,
`gate_pulse_ms`, `telemetry_interval_s`. An unparseable value is ignored (the previous
value stays).

**Resetting runtime config**: `POST /config` can set a value back to anything, but
there is no "reset to default" endpoint. To wipe all NVS settings:

```
espflash erase-parts nvs
```

(after which the compile-time defaults apply). `espflash erase-flash` wipes the whole
flash including bootloader and partition table — reflash everything afterwards.

---

## HTTP API

The HTTP server always runs on port 80 (even while WiFi is still connecting). `POST /open`
and `POST /close` are **fire-and-forget**: they queue the command and reply `OK`
immediately; the gate moves asynchronously (check `/status` for the real state).

| Method | Path | Auth | Response |
|---|---|---|---|
| `POST` | `/open` | `X-Api-Key` if set | `OK` (queues open) |
| `POST` | `/close` | `X-Api-Key` if set | `OK` (queues close) |
| `GET` | `/status` | — | `open` / `closed` / `opening` / `closing` / `stopped` / `error` |
| `GET` | `/config` | — | Effective config as JSON (see above) |
| `POST` | `/config` | `X-Api-Key` if set | `OK` |
| `POST` | `/ota` | `X-Api-Key` if set | reboots (or `OTA failed`, HTTP 500) |

With `HTTP_API_KEY` set, `/open`, `/close`, `/config` POST and `/ota` return HTTP **401**
without a valid `X-Api-Key` header:

```
curl -X POST -H "X-Api-Key: <HTTP_API_KEY>" "http://<device-ip>/open"
```

> `GET /config` is intentionally **not** authenticated and reveals the current
> `http_api_key` in plain text. Treat the device IP as trusted-network-only, or run it
> behind a reverse proxy / firewall that blocks unauthenticated `GET /config`.

---

## MQTT protocol

The device publishes with **QoS 1 (at-least-once) and `retain=true`**, so a late
subscriber immediately sees the last state. Commands are subscribed with **QoS 0**.

| Topic (default) | Direction | Payload |
|---|---|---|
| `gate/command` | receive | `open` or `close` (exact match) |
| `gate/status` | publish | `open` / `closed` / `opening` / `closing` / `stopped` / `error` |
| `gate/availability` | publish | `online` / `offline` (LWT: `offline`) |
| `gate/battery` | publish | `0`–`100` (%) |
| `gate/battery_voltage` | publish | e.g. `12.30` (V, 2 decimals) |
| `gate/obstacle` | publish | `on` / `off` |
| `gate/fault` | publish | `on` / `off` |

- On (re)connect the device publishes `online`, status, obstacle, fault **and** the
  Home Assistant discovery configs. Battery/voltage are refreshed **only** every
  `TELEMETRY_INTERVAL_S` (not on connect).
- The battery percentage is the **median of 8 ADC samples** scaled by
  `BATTERY_DIVIDER_RATIO`, then mapped linearly between `BATTERY_EMPTY_MV` and
  `BATTERY_FULL_MV` and clamped to 0–100. On an ADC error `0` is published and the
  voltage topic is skipped.
- Raw examples with `mosquitto`:

```
mosquitto_pub -h <broker> -u <user> -P <pass> -t gate/command -m open
mosquitto_pub -h <broker> -u <user> -P <pass> -t gate/command -m close
mosquitto_sub -h <broker> -u <user> -P <pass> -t gate/status
mosquitto_sub -h <broker> -u <user> -P <pass> -t gate/battery
mosquitto_sub -h <broker> -u <user> -P <pass> -t gate/battery_voltage
mosquitto_sub -h <broker> -u <user> -P <pass> -t gate/obstacle
mosquitto_sub -h <broker> -u <user> -P <pass> -t gate/fault
```

### Home Assistant (recommended)

The discovery configs are published (retained) on every (re)connect, so Home
Assistant automatically creates:

- a **Cover** entity `cover.gate` — state `open` / `closed` / `opening` / `closing`
  with open/close buttons,
- a **Battery** sensor `sensor.gate_battery` — charge level in %,
- a **Voltage** sensor `sensor.gate_battery_voltage` — battery voltage in V,
- a **Binary sensor** `binary_sensor.gate_obstruction` (class `safety`) — `on` while
  the gate opening is blocked,
- a **Binary sensor** `binary_sensor.gate_fault` (class `problem`) — `on` on battery
  lockout or a sensor fault (`error` state).

Because the device is always on, the cover buttons work **instantly**. The state topic
updates immediately after each command and periodically via telemetry.

---

## Build prerequisites (toolchain)

- [Rustup](https://rustup.rs/)
- [espup](https://github.com/esp-rs/espup) — installs the `esp` toolchain
  (defined in `rust-toolchain.toml`) and ESP-IDF
- `espflash` (v4+) and `ldproxy`:
  ```
  cargo install espflash ldproxy
  ```
- On Windows also Git + a C toolchain (see the
  [esp-idf-template prerequisites](https://github.com/esp-rs/esp-idf-template#prerequisites)).
  The project is built inside WSL (`/mnt/c/Projekty/esp32-gate-opener`).

## Building and flashing

Set up the ESP toolchain (first time only):

```
espup install
```

Build the release binary:

```
cargo build --release
```

Flash the firmware (board connected over USB):

```
espflash flash --monitor target\xtensa-esp32-espidf\release\esp32-gate-opener.exe
```

> Because of the `runner` entry in `.cargo/config.toml`, `cargo run --release` also
> flashes (including `--erase-parts otadata`) and opens the serial monitor.

### OTA partition table

The project ships a **two-slot OTA partition table** (`partitions.csv` + `sdkconfig.defaults`)
instead of the default single-slot table:

```
# Name,   Type, SubType, Offset,  Size,     Flags
nvs,      data, nvs,     ,        0x6000,
otadata,  data, ota,     ,        0x2000,
phy_init, data, phy,     ,        0x1000,
ota_0,    app,  ota_0,   ,        1700K,
ota_1,    app,  ota_1,   ,        1700K,
```

The **`otadata` partition is erased on every USB flash** (`--erase-parts otadata` in
the `runner`), so after a USB reflash the bootloader deterministically boots slot
`ota_0`. This is also handy to recover if the OTA metadata ever gets into a
surprising state.

> **Build-system note**: the ESP-IDF build resolves `CONFIG_PARTITION_TABLE_CUSTOM_FILENAME`
> relative to its own project dir (the `esp-idf-sys` build output), not the crate root.
> `.cargo/config.toml` therefore copies `partitions.csv` into that dir via the
> `ESP_IDF_GLOB_BASE` / `ESP_IDF_GLOB_PARTITION_CSV` env vars. Keep them in place or a
> fresh build fails with "`partitions.csv` ... missing and no known rule to make it".

### OTA update over HTTP

Flash a new firmware image without touching the USB cable. First produce a flashable
**application image** from the built ELF (espflash ≥ 4, `--chip` is required):

```
espflash save-image --chip esp32 target\xtensa-esp32-espidf\release\esp32-gate-opener.exe gate.bin
```

> Do **not** pass `--merge` — that produces a full flash image (bootloader + partition
> table + app, padded to the flash size) which is only for USB flashing, not for OTA.
> Without `--merge` you get the ~1.2 MB app-only image that the `/ota` endpoint writes
> into the inactive slot.

Then upload it to the device:

```
curl -X POST -H "X-Api-Key: <HTTP_API_KEY>" "http://<device-ip>/ota" --data-binary @gate.bin
```

The device streams the body into the inactive OTA slot, validates the image, sets it
as the boot partition and reboots. The endpoint returns 401 without the key and 500
if the write fails.

### OTA and rollback

`sdkconfig.defaults` enables `CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE`. With that option
an OTA-updated slot is marked **"pending verification"** until the new firmware
confirms itself. In this firmware the confirmation happens automatically at the very
start of `main()` (`mark_running_slot_valid()`):

- If the new firmware boots and gets past basic init, it is confirmed and **stays**
  the running slot across subsequent reboots.
- If the new firmware crashes before that confirmation (early boot failure, panic in
  init), the bootloader **reverts to the previous slot** on the next boot.

So a good update survives power cycles, and a broken update self-heals on reboot.
(The confirmation is skipped only if the OTA subsystem reports an error, which is
logged as a warning.)

> Development note: USB-flashing the same device over `cargo run`/`espflash flash`
> erases `otadata`, which resets the OTA state back to slot `ota_0`. That is intended
> for development but means an OTA slot chosen by a previous HTTP update is forgotten.

### Running the unit tests

The pure logic (status/sensor mapping, battery %, obstacle level) lives in
`src/pure/` with host-side `#[cfg(test)]` tests (in `src/pure/tests.rs`). Run them
on a host toolchain:

```
cargo test --lib
```

---

## Repository layout

```
esp32-gate-opener/
├── src/
│   ├── main.rs            Boot, WiFi, async task setup
│   ├── config.rs          Compile-time config (env!) + constants
│   ├── config_storage/    Runtime config in NVS (overrides defaults)
│   ├── gate/              Motion sequences, fail-open safety, lamp/relay control
│   ├── http/              HTTP server, endpoints, `X-Api-Key` auth
│   ├── ota.rs             OTA flashing over HTTP
│   ├── homeassistant/     MQTT client, telemetry publishes, HA discovery
│   ├── state/             Shared atomics (status, command, fault, obstacle, battery)
│   └── pure/              Pure logic + host-side unit tests
├── .cargo/config.toml     Build target, runner, compile-time `[env]` config
├── partitions.csv         Two-slot OTA partition table
├── sdkconfig.defaults     ESP-IDF Kconfig overrides (partition table, rollback)
├── espflash.toml          espflash flash settings (partition table)
└── Cargo.toml             Crate + release profile (LTO, opt-size, panic=abort)
```

Each module folder (`config_storage/`, `gate/`, `http/`, `homeassistant/`, `state/`,
`pure/`) follows a **one file per function** layout: shared state/constants live in
`mod.rs`, each function has its own file, and functions are re-exported from `mod.rs`.

The release profile is tuned for a production image: `lto = "fat"`,
`codegen-units = 1`, `opt-level = "s"` and `panic = "abort"` — the resulting app
image is roughly 1.2 MB, well inside the 1700 K slot.

---

## Safety and deployment notes

- **Obstacle sensor scope**: the obstruction check runs while the gate is moving under
  a `close` command (i.e. the ESP32 is awake and watching). If the gate is operated
  from the physical remote/button, the ESP32 obstacle sensor is **not** involved — so
  keep the gate controller's own safety inputs (photocell inputs) wired to the
  through-beam receiver as well, if the controller supports them.
- **Authentication**: anyone who can reach the MQTT broker or the HTTP endpoint
  can operate the gate. Set `MQTT_USERNAME`/`MQTT_PASSWORD` in `.cargo/config.toml`
  and an `HTTP_API_KEY` (used by `/open`, `/close`, `/ota` and `/config`). Put the
  device on an isolated IoT VLAN.
- **Battery lockout**: below `BATTERY_MIN_PCT` the gate refuses all motion commands.
  This preserves the battery for critical loads, but it also means a gate that is
  closed stays closed while the battery is low — if that is a safety problem for your
  installation, set the threshold low or keep the physical remote/button working.
- **Keep the manual button/remote**: always keep the physical button or remote
  working as a fallback.
- **Surge protection**: if signal wires run near power lines, use optoisolation and
  a separate relay supply.
- **Relay protection**: put a flyback diode across the relay coil (most module boards
  already have one).
- **Battery low-voltage cutoff**: use a charge controller with low-voltage disconnect
  to protect the battery from deep discharge — especially important now that the
  device is always on.

## Limitations

- **Single-slot command queue**: the device holds at most one pending command — the
  last MQTT/HTTP command wins and commands are not queued.
- **No `stop` command**: the supported commands are `open` and `close` only. To stop a
  running motion, send the opposite command (it aborts the current action and starts
  the new one).
- **`stopped` state and Home Assistant**: when the gate sits between the two end
  sensors (e.g. interrupted mid-travel), the status is `stopped`. The HA cover has no
  mapping for `stopped`, so the cover keeps its last known state until the next
  `open`/`closed`/`opening`/`closing` message.
- **Obstacle only on close**: the beam is only polled during closing. An obstruction
  does not block an opening motion.
- **Battery freshness**: the battery % is updated every `TELEMETRY_INTERVAL_S`; the
  lockout and reported percentage use that last sample (starts at 100 % at boot).
- **`GET /config` is open**: it is unauthenticated and includes the API key; see the
  HTTP section.
- **MQTT topics are fixed at compile time** and the discovery root is hardcoded to
  `homeassistant/...`. Change them via the `[env]` section if you use a different
  discovery prefix.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| No Wi-Fi connection | Wrong SSID/password | Fix values in `.cargo/config.toml`, rebuild |
| MQTT command ignored | Broker URL wrong | Check `MQTT_BROKER`; check serial log on boot |
| Gate moves but lamp off | Wrong pin or MOSFET wiring | Check GPIO27/GPIO14 wiring and drivers |
| Lamp wrong color | Open/closed sensors swapped | Swap GPIO25/GPIO26 wiring or magnets |
| `open` skips although gate is closed | Sensor logic inverted | Reed switches should close to GND at the end position |
| Battery level wrong | Wrong divider values | Set `BATTERY_DIVIDER_RATIO` and full/empty voltages for your divider and chemistry |
| Battery level stuck at 100%/0% | Wrong `BATTERY_FULL_MV`/`BATTERY_EMPTY_MV` | Calibrate with a multimeter |
| Status shows `error` | Both reed switches low at once | Check GPIO25/GPIO26 wiring; one should be high |
| Gate refuses to move | Battery below `BATTERY_MIN_PCT` | Check `gate/fault`; wait for charge or lower the threshold |
| Gate opens by itself | Interference on GPIO line | Shorten wires, add pull resistors, use shielded cable |
| Battery drains in a day | Panel too small for 24/7 | Use 20–30 W panel and 12 Ah battery (see power budget) |
| No MQTT reconnect after WiFi drop | MQTT client internal backoff | Reconnect is automatic (esp-mqtt); check broker reachability |
| `POST /ota` returns 401 | Wrong/missing `HTTP_API_KEY` | Send an `X-Api-Key: <HTTP_API_KEY>` header |
| OTA `save-image` fails | Missing `--chip esp32` | espflash ≥ 4 requires `--chip` |
| Update reverts after a power cycle | OTA slot never confirmed | Confirm it boots the new firmware to the mark-valid point; check the serial log for "Failed to mark OTA slot valid" |
| Runtime config won't reset | NVS holds stale values | `espflash erase-parts nvs`, or set the value back via `POST /config` |
| `espflash` config error about `partition_table` | espflash 4.x schema | Keys live under `[idf_format_args]`; see `espflash.toml` |
