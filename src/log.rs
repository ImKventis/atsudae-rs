
// Created by gg@glinux 28/11/2022 04:48PM

// A very simple logger
// I decided to use a structure with methods rather than functions because every function will need level and start_t
// putting it into a structure just makes sense

use std::fs::File;

static NO_COL: &'static str = "\x1b[0m";
static CYAN: &'static str = "\x1b[0;36m";
static GREEN: &'static str = "\x1b[0;32m";
static YELLOW: &'static str = "\x1b[0;33m";
static RED: &'static str = "\x1b[0;31m";

pub struct Logger {
    start_t : std::time::SystemTime,
    /// 1 = ERR, 4 = DEBUG, 0 = NONE
    pub level: u8,
    pub file_path: String,
    file: Option<File>
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            start_t: std::time::SystemTime::now(),
            level: 4,
            file: None,
            file_path: String::from(""),
        }
    }
}

impl Logger {
    pub fn new(level: u8) -> Self {
        let l = Logger {  level, ..Logger::default()};
        l
    }

    fn log(&mut self, col: &'static str, level_str: &str, message: &str) {
        use std::io::Write;

        let timesince = match self.start_t.elapsed() {
            Ok(v) => {v.as_secs()}
            Err(_e) => { 0 }
        };

        // Gotta format twice to not include col in file
        // let out = format!("{}{}{} [{}ms] - {}", col, level_str, NO_COL, timesince.to_string(), message);

        println!("{}{}{} [ {}s ] - {}", col, level_str, NO_COL, timesince.to_string(), message);

        // File printing now

        let outfile = match &mut self.file {
            None => {return; /* Just forget it */}
            Some(f) => {f}
        };

        match outfile.write(format!("{} [{}ms] - {}\n", level_str, timesince.to_string(), message).as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                self.file = None; // Remove file first to prevent a loop
                self.error("Failed to write to log file, file logging will be disabled");
                self.debug(&format!("Write_all error: {:?}", e));
            }
        };
    }

    pub fn debug(&mut self, message: &str) {
        if self.level >= 4 {
            self.log(CYAN,"DEBUG", message);
        }
    }
    pub fn info(&mut self, message: &str) {
        if self.level >= 3 {
            self.log(GREEN,"INFO", message);
        }
    }
    pub fn warn(&mut self, message: &str) {
        if self.level >= 2 {
            self.log(YELLOW,"WARN", message);
        }
    }
    pub fn error(&mut self, message: &str) {
        if self.level >= 1 {
            self.log(RED, "ERROR", message);
        }
    }
    // Used to send message that are the same priority as error but not an error, shows on all the log level except log level 0 (silent)
    pub fn message(&mut self, message: &str) {
        if self.level > 0 {
            self.log(NO_COL, "NOTICE:", message);
        }
    }

    pub fn change_file(&mut self, path: &String) {
        use std::io::Write;

        self.debug(&format!("Attempting to change logging file to {}", path));
        let mut f = match File::options().append(true).write(true).create(true).open(&path) {
            Ok(v) => {v}
            Err(e) => {
                self.error(&format!("Could not change the logging file to {}", path));
                self.debug(&format!("OS Error {:?}", e));
                return
            }
        };
        match f.write_all("---- NEW SESSION ----\n".as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                self.error("Failed to write to the new logging file");
                self.debug(&format!("OS Error {:?}", e));
                return
            }
        }
        self.file = Some(f);
        self.file_path = String::from(path);
        self.info(&format!("Changed logging file to {}", path));
    }

}