
// Created by gg@gglaptop 28/11/2022 11:33AM

#[cfg(windows)]
compile_error!("This package will only run on UNIX and Linux Systems due to the use of linux driver interaction and lm_sensors");
mod sysfs;
mod log;
mod sensors;

#[derive(Debug)]
struct Config {
    log_level: u8,
    boost: bool,
    file_log: String,
    max_fail: u8,
    stdio: bool
}

impl Default for Config {
    fn default() -> Self {
        Config {
            log_level: 2,
            boost: false,
            file_log: "/var/log/atsudae.log".to_string(),
            max_fail: 5,
            stdio: false
        }
    }
}

fn load_cfg(logger: &mut log::Logger) -> Config {
    use std::fs::File;
    use std::io::Read;

    if !std::path::Path::new("/etc/atsudae.conf").exists() {
        return Config::default();
    }

    let mut cfgfile: File = match File::open("/etc/atsudae.conf") {
        Ok(v) => {v}
        Err(e) => {
            logger.error("Failed to open /etc/atsudae.conf for reading");
            logger.debug(&format!("OS Error {:?}", e));
            return Config::default();
        }
    };

    let mut buf: Vec<u8> = Vec::with_capacity(100);
    match cfgfile.read_to_end(&mut buf) {
        Ok(_) => {}
        Err(e) => {
            logger.error("Failed to open /etc/atsudae.conf for reading");
            logger.debug(&format!("Read Error {:?}", e));
            return Config::default();
        }
    };

    let output = match String::from_utf8(buf) {
        Ok(v) => {v}
        Err(e) => {
            logger.error("/etc/atsudae.conf contains invalid UTF-8 bytes");
            logger.debug(&format!("UTF-8 Error {:?}", e));
            return Config::default();
        }
    };

    let mut config = Config::default();

    for s in output.split(";") {
        let s = s.trim();
        // Comments
        if s.starts_with("#") {
            continue;
        }
        if s.starts_with("LOGLEVEL=") {
            let con = s.split_at("LOGLEVEL=".len());
            config.log_level = match con.1.parse::<u8>() {
                Ok(v) => {v},
                Err(e) => { logger.error("Failed to convert LOGLEVEL to an u8, defaulting to 2"); logger.debug(&format!("Parse error {:?}", e)); 2 }
            }
        }
        if s.starts_with("MAXFAIL=") {
            let con = s.split_at("MAXFAIL=".len());
            config.max_fail = match con.1.parse::<u8>() {
                Ok(v) => {v},
                Err(e) => { logger.error("Failed to convert MAXFAIL to an u8, defaulting to 5"); logger.debug(&format!("Parse error {:?}", e)); 2 }
            }
        }
        if s.starts_with("BOOST=") {
            let con = s.split_at("BOOST=".len());
            config.boost = match con.1.parse::<bool>() {
                Ok(v) => {v}
                Err(e) => { logger.error("Failed to convert BOOST to a bool, defaulting to false"); logger.debug(&format!("Parse error {:?}", e)); false }
            }
        }
        if s.starts_with("STDIO=") {
            let con = s.split_at("STDIO=".len());
            config.stdio = match con.1.parse::<bool>() {
                Ok(v) => {v}
                Err(e) => { logger.error("Failed to convert STDIO to a bool, defaulting to false"); logger.debug(&format!("Parse error {:?}", e)); false }
            }
        }
        if s.starts_with("LOGFILE=") {
            let con = s.split_at("LOGFILE=".len());
            config.file_log = con.1.to_owned()
        }
    };

    config

}

// Boosts cpu when cold, turns it off when its hot
fn boost_loop(logger: &mut log::Logger, sensors: &lm_sensors::LMSensors, intel: bool) -> u8 {

    let ctemp = match sensors::get_temp(sensors) {
        Ok(v) => {v}
        Err(e) => {
            let _ = sysfs::set_status(intel, 0); // last ditch effort tbh
            logger.error("There was an error getting the cpu temperature");
            logger.debug(&format!("IO Error: {:?}", e));
            return 1;
        }
    };

    let cstatus = match sysfs::get_status(intel) {
        Ok(v) => {v}
        Err(e) => {
            logger.error("There was an error getting the current boost status");
            logger.debug(&format!("IO Error: {:?}", e));
            return 1;
        }
    };

    if cstatus { // Boost is on
        if ctemp > 80 { // Thats hot
            logger.debug("CPU is too hot, turning boost off now");
            match sysfs::set_status(intel, 0) {
                Ok(_) => {}
                Err(e) => {
                    logger.error("Failed to set the status!");
                    logger.warn("Boost cannot be turned off!");
                    logger.debug(&format!("IO Error: {:?}", e));
                    return 1;
                }
            }
        }else if ctemp > 65 { // comfortable
            logger.debug("CPU is OK, leaving boost how it is");
        }else if ctemp > 40 { // chilly
            logger.debug("CPU is cold, leaving boost how it is");
        }
    }else { // Boost is not on
        if ctemp > 80 { // Thats hot
            logger.debug("CPU is too hot, leaving boost how it is");
        }else if ctemp > 65 { // comfortable
            logger.debug("CPU is ok, turning boost on now");
            match sysfs::set_status(intel, 1) {
                Ok(_) => {}
                Err(e) => {
                    logger.error("Failed to set the status, boost is still off");
                    logger.debug(&format!("IO Error: {:?}", e));
                    return 1;
                }
            }
        }else if ctemp > 40 {
            match sysfs::set_status(intel, 1) {
                Ok(_) => {}
                Err(e) => {
                    logger.error("Failed to set the status, boost is still off");
                    logger.debug(&format!("IO Error: {:?}", e));
                    return 1;
                }
            }
        }
    }

    0
}

// Just to ensure CPU does not boost eg when on battery
fn non_boost_loop(logger: &mut log::Logger, intel: bool) {
    // logger.info("Non boost loop ran once");
    let cstatus = match sysfs::get_status(intel) {
        Ok(v) => {v}
        Err(e) => {
            logger.error("There was an error getting the current boost status");
            logger.debug(&format!("IO Error: {:?}", e));
            return;
        }
    };

    if cstatus {
        match sysfs::set_status(intel, 0) {
            Ok(_) => {}
            Err(e) => {
                logger.error("Failed to set the status!");
                logger.warn("Boost is still on!");
                logger.debug(&format!("IO Error: {:?}", e));
                return;
            }
        }
        logger.debug("Boost status has been disabled");
    }
}

fn main() {
    use std::sync;

    let mut logger: log::Logger = log::Logger::new(4);

    let cfg_main = sync::Arc::new(sync::Mutex::new(load_cfg(&mut logger)));
    {
        // .unwrap cuz this should never fail
        let cfg = cfg_main.lock().unwrap();
        logger.level = cfg.log_level;
        logger.debug(&format!("Using Config: {:?}", cfg));
        logger.change_file(&cfg.file_log);
    }

    let sensors = lm_sensors::Initializer::default().initialize();
    let intel = match sysfs::get_bdriver() {
        Ok(v) => {v}
        Err(e) => {
            logger.error("Failed to detect what cpu driver is being used.. Program cannot continue at all");
            logger.debug(&format!("IO Error: {:?}", e));
            std::process::exit(1);
        }
    };

    let intel = if intel.starts_with("intel_pstate") { true } else { false };
    if intel {
        logger.warn("Intel CPU detected, special cases are going to be used");
    }

    // Originally used a channel for this but decided to use a Arc<Mutex> because channel was not providing
    // much over Arc<Mutex>
    // Listen to stdio for key strokes
    logger.message("Now listening for commands");

    {
        if cfg_main.lock().unwrap().stdio {
            let cfg_stdio = sync::Arc::clone(&cfg_main);
            std::thread::spawn(move || {
                loop {
                    let mut inn = String::new();
                    match std::io::stdin().read_line(&mut inn) {
                        Ok(_) => {}
                        Err(_e) => {continue;}
                    }
                    match inn.trim() {
                        // Dont *really* need to match this Result as its impossible for the main thread to
                        // panic without also killing this thread in the process. I would of used match
                        // if there was more threads involved.
                        "boost" => cfg_stdio.lock().unwrap().boost = true,
                        "noboost" => cfg_stdio.lock().unwrap().boost = false,
                        "clear" => println!("\x1B[2J"),
                        "help" => println!("   boost - Use the boost loop\n   noboost - Use the no boost loop\n   clear - Clears the screen"),
                        _ => {}
                    }
                }
            });
        }
    }

    let mut failed_loops: u8 = 0;
    let mut poisoned = false;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1500));

        let mut cfg = match cfg_main.lock() {
            Ok(v) => {
                v
            }
            Err(e) => {
                // I feel safe using the Poisoned Mutex, its incredibly unlikely that anything in the stdio thread will
                // panic or poison the mutex enough for it to be considered "unpredictable"
                if !poisoned {
                    logger.error("Config is poisoned, Will continue but a restart is advised");
                    poisoned = true;
                }
                e.into_inner()
            }
        };

        if failed_loops >= cfg.max_fail && failed_loops <= 10 {
            logger.warn("The boost loop has failed to complete too many times, disabling boost until manually overwritten");
            cfg.boost = false;
            failed_loops = 0;
        }

        if cfg.boost && sensors.is_ok() {
            failed_loops += boost_loop(&mut logger, sensors.as_ref().unwrap(), intel);
        }else if cfg.boost && sensors.is_err() { // User ran boost in separate thread but sensors are still error
            logger.warn("Boost has been enabled but sensors are still inaccessible, disabling boost");
            cfg.boost = false;
        } else {
            non_boost_loop(&mut logger, intel);
        }
    };
}

