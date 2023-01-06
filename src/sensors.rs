
// Created by gg@glinux 28/11/2022 05:22PM

// Decided to go for koutheir (https://github.com/koutheir) lm_sensors crate rather than using .so
// because it is not worth the effort

use lm_sensors;
use lm_sensors::prelude::SharedChip;

/// Some common sensors I see on linux systems
static KNOWN_SENSORS: [&'static str;2] = ["k10temp-pci", "coretemp"];

/// Get the cpu temperature using lm_sensors
// note: This is equally annoying as it was in C but I cant blame it its just a C wrapper after all
pub fn get_temp(sensors: &lm_sensors::LMSensors) -> std::io::Result<u8> {

    let mut count: u8 = 0;
    let mut total: f64 = 0.0;

    for chip in sensors.chip_iter(None) {
        let chip_name = match chip.name() {
            Ok(v) => v,
            Err(_) => continue
        };
        for i in KNOWN_SENSORS.iter() {
            if !chip_name.starts_with(i) {
                continue
            }
            // Found a valid chip
            for feature in chip.feature_iter() {
                if feature.kind() != Some(lm_sensors::feature::Kind::Temperature) {
                    continue
                }
                for subfeature in feature.sub_feature_iter() {
                    let value = match subfeature.value() {
                        Ok(v) => v,
                        Err(_e) => continue
                    };
                    let coretmp = match value {
                        lm_sensors::Value::TemperatureInput(v) => {v},
                        _ => continue
                    };
                    total += coretmp;
                    count += 1;
                }
            }
        }
    }
    if count == 0 {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No compatible sensors found"))
    }
    Ok(total as u8 / count)
}