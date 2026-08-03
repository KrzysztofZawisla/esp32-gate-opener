use esp_idf_hal::reset;
use esp_idf_svc::ota::EspOta;
use log::info;

use anyhow::Result;

/// Streams an HTTP request body into the inactive OTA slot, validates the image,
/// sets it as the boot partition and restarts the device. `read_chunk` must read
/// the body into `out` and return the number of bytes read (0 = end of body).
pub fn flash_ota<E>(read_chunk: &mut dyn FnMut(&mut [u8]) -> Result<usize, E>) -> Result<()>
where
    E: std::error::Error + Send + Sync + 'static,
{
    let mut ota = EspOta::new()?;
    let mut update = ota.initiate_update()?;

    let mut buf = [0u8; 4096];
    loop {
        let n = read_chunk(&mut buf)?;
        if n == 0 {
            break;
        }
        update.write(&buf[..n])?;
    }

    update.complete()?;
    info!("OTA update complete, restarting");
    reset::restart();
}
