use esp_idf_hal::reset;
use esp_idf_svc::ota::EspOta;
use log::info;

use anyhow::Result;

/// Streams an HTTP request body into the inactive OTA slot, validates the image,
/// sets it as the boot partition and restarts the device. `read_chunk` must read
/// the body into `buffer` and return the number of bytes read (0 = end of body).
pub fn flash_ota<ReadError>(
    read_chunk: &mut dyn FnMut(&mut [u8]) -> Result<usize, ReadError>,
) -> Result<()>
where
    ReadError: std::error::Error + Send + Sync + 'static,
{
    let mut ota = EspOta::new()?;
    let mut update = ota.initiate_update()?;

    let mut buffer = [0u8; 4096];
    loop {
        let bytes = read_chunk(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        update.write(&buffer[..bytes])?;
    }

    update.complete()?;
    info!("OTA update complete, restarting");
    reset::restart();
}
