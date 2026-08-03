use esp_idf_sys::nvs::{EspDefaultNvsPartition, EspNvs};
use log::{info, warn};

use super::{load_all, restore_defaults, NVS, NVS_NAMESPACE};

pub fn init(partition: EspDefaultNvsPartition) {
    restore_defaults();

    match EspNvs::new(partition, NVS_NAMESPACE, true) {
        Ok(nvs) => {
            info!("Runtime config opened in NVS");
            *NVS.lock().unwrap() = Some(nvs);
            load_all();
        }
        Err(error) => warn!("Failed to open NVS runtime config: {error}; using defaults"),
    }
}
