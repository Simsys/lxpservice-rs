extern crate chrono;

use chrono::prelude::*;
use log::{Level, LevelFilter, Metadata, Record};
use std::io::prelude::*;

#[derive(Clone)]
struct AppLogger {
    max_level: LevelFilter,
    app_name: String,
}

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Write to log File
            let file_name = self.app_name.clone() + ".log";
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(&file_name)
                .unwrap();

            let local = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let s = format!("{} {} - {}\n", &local, record.level(), record.args());
            if let Err(e) = file.write_all(s.as_bytes()) {
                eprintln!("Couldn't write to logfile: {}", e);
                std::process::exit(1);
            }

            // Write to stdio or stderror
            match record.level() {
                Level::Error => {
                    eprintln!("{} - {}", record.level(), record.args());
                    eprintln!("aborting...");
                    std::process::exit(1);
                }
                Level::Info => println!("{}", record.args()),
                _ => println!("{} - {}", record.level(), record.args()),
            }
        }
    }

    fn flush(&self) {}
}

pub fn init(app_name: &str, level: LevelFilter) { 
    let logger = AppLogger {
        app_name: app_name.to_string(),
        max_level: level,
    };
    match log::set_boxed_logger(std::boxed::Box::new(logger)) {
        Ok(_) => log::set_max_level(level),
        Err(e) => {
            eprintln!("Couldn't initialize logger: {}", e);
            std::process::exit(1);
        },
    }
}
