use std::fs::{OpenOptions, File};
use std::str::FromStr;
use std::fmt::Display;
use chrono::DateTime;
use serde::{Serialize, Deserialize};
use regex::Regex;
use log;

use crate::decompressors::decompress::CompressionFormat;

// log parser config
#[derive(Serialize, Deserialize, Debug)]
pub struct ParserConfig {
    timestamp_format: String,
    event_filter: String,
    pub compression: CompressionFormat,
    pub message_pattern: String,
    pub logfile_pattern: String
}


impl ParserConfig {
    pub fn new(compression: &str) -> ParserConfig {
        return ParserConfig{
            timestamp_format: String::new(),
            event_filter: String::new(),
            compression: CompressionFormat::from_str(compression).unwrap(),
            message_pattern: String::new(),
            logfile_pattern: String::new()
        };
    }

    pub fn compile_message_pattern(&self) -> Regex {
        let pattern = match Regex::new(&self.message_pattern) {
            Ok(p) => p,
            Err(e) => panic!("Invalid logline pattern, error {:?}", e),
        };
        return pattern;
    }

    pub fn read_from_file(path: &str) -> ParserConfig {
        let file = File::open(path).unwrap();
        let config: ParserConfig = match serde_yaml::from_reader(file) {
            Ok(c) => c,
            Err(e) => panic!("Error while reading parser config. Error: {}", e)
        };
        return config;
    }

    pub fn generate_template(config_path: String) {
        log::info!("Generatig new parser configuration file to path {}", config_path);
        let config = ParserConfig::new("none");
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(config_path)
            .expect("Couldn't open file");
        serde_yaml::to_writer(f, &config).unwrap();
    }

    pub fn read_timestamp(&self, datestring: &str) -> i64 {
        if self.timestamp_format.is_empty() {
            let unix = match datestring.parse::<i64>() {
                Ok(r) => r,
                Err(e) => panic!("Could not parse unix timestamp, error {}", e)
            };
            return unix;
        }

        let date = match DateTime::parse_from_str(datestring, &self.timestamp_format) {
            Ok(dt) => dt,
            Err(e) => panic!("Could not parse datetime, error {:?}", e)
        };
        date.timestamp()
    }

    pub fn filter_event(&self, event: &str) -> bool {
        // make this regex pattern! at some point maybe
        if !self.event_filter.is_empty() && event.contains(&self.event_filter) {
            return true;
        }
        false
    }
}


impl Display for ParserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Timestamp format: {}, Compression format: {:?}, Message pattern: {}, Logfile pattern: {}",
            self.timestamp_format,
            self.compression,
            self.message_pattern,
            self.logfile_pattern
        )
    }
}
