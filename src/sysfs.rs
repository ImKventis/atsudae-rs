
// Created by gg@gglaptop 28/11/2022 11:35AM

// Module for operations that take place in /sys/devices/system/cpu/
// Eg enable boost, check boost status, etc

use std::path::Path;
use std::fs::File;
use std::io;
use io::Read;

/// scaling_driver location doesnt change across tested linux operating Systems
static DRIVER_PATH: &'static str = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_driver";

/// Gets the current driver by reading /sys/devices/system/cpu/
pub fn get_bdriver() -> io::Result<String> {

    if !Path::new(DRIVER_PATH).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File was not found"));
    }

    let mut file: File = File::open(DRIVER_PATH)?;

    // Using byte array over a Vector as the sysfs reports files being 4096 in length
    // read_to_end will therefore resize the Vector to 4096 as the fs reports
    // ? this is way way too much although in reality 4Kb instead of 13B of ram isn't going to hurt anyone
    let mut buf : [u8;13] = [0;13];
    let read: usize = file.read(&mut buf)?;
    // Using str::from_utf8 instead of String::from_utf8 as String::from_utf8 requires a Vector instead of byte array
    match std::str::from_utf8(&buf[0..read]) {
        Ok(s) => Ok(s.trim_end().to_owned()),
        Err(_e) => Err(io::Error::new(io::ErrorKind::InvalidData, "File included non-UTF8 bytes"))
    }
}

/// Gets the current boost status by reading the boost file
pub fn get_status(intel: bool) -> io::Result<bool> {
    let boost_path: &str = if intel {"/sys/devices/system/cpu/intel_pstate/no_turbo"} else {"/sys/devices/system/cpu/cpufreq/boost"};
    // let ON: bool = intel;

    if !Path::new(boost_path).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File was not found"));
    }

    let mut file: File = File::open(boost_path)?;

    // Should only need one byte as the file will contain either 1 or 0
    let mut buf: [u8; 1] = [0;1];
    let read: usize = file.read(&mut buf)?;
    let content: &str = match std::str::from_utf8(&buf[0..read]) {
        Ok(s) => s,
        Err(_v) => return Err(io::Error::new(io::ErrorKind::InvalidData, "File included non-UTF8 bytes"))
    };
    let v = match content.parse::<u8>() {
        Ok(v) => v != 0,
        Err(_e) => return Err(io::Error::new(io::ErrorKind::InvalidData, "File did not include a number. reading was inaccurate"))
    };

    // Reverse for intel
    Ok(if intel { !v } else { v })
}

pub fn set_status(intel: bool, value: u8) -> io::Result<()> {
    use io::Write;
    let boost_path: &str = if intel {"/sys/devices/system/cpu/intel_pstate/no_turbo"} else {"/sys/devices/system/cpu/cpufreq/boost"};

    if !Path::new(boost_path).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File was not found"));
    }

    let mut file: File = File::create(boost_path)?;
    // Had to change value into a string before writing
    // Tried using bytes and char bytes but those didnt work at all
    let value = if intel { if value == 1 {0} else { 1 } } else { value };
    file.write_all(value.to_string().as_bytes())?;
    Ok(())
}
