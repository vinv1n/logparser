use std::vec::Vec;
use std::path::PathBuf;
use std::str;
use std::str::FromStr;
use glob::glob;
use serde::Serialize;
use log::{info, warn};
use std::io::ErrorKind;
use std::fmt::Display;

use crate::parser_config::parser_config::ParserConfig;
use crate::decompressors::decompress::{Decompressor, Decompress};


#[derive(Serialize, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Warn,
    Critical,
    Error,
    Unknown
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(LogLevel::Info),
            "warning" => Ok(LogLevel::Warning),
            "error" => Ok(LogLevel::Error),
            "debug" => Ok(LogLevel::Debug),
            "critical" => Ok(LogLevel::Critical),
            "warn" => Ok(LogLevel::Warn),
            _ => Ok(LogLevel::Unknown),
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // maybe someday make better formating for this
        write!(f, "{:?}", self)
    }
}


#[derive(Debug)]
struct FileEntry{
    path: PathBuf
}

impl FileEntry {
    fn new(path: PathBuf) -> FileEntry {
        return FileEntry{path};
    }
}


#[derive(Serialize, Debug)]
pub struct Event {
    timestamp: i64,
    message: String,
    level: LogLevel
}

impl Event {
    // TODO: implement display for event
    fn new(timestamp: i64, message: &str, loglevel: &str) -> Event {
        return Event{
            timestamp: timestamp,
            message: String::from(message),
            level: LogLevel::from_str(loglevel).unwrap()
        };
    }

    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[allow(dead_code)]
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }
}


impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} - {}", self.timestamp, self.level, self.message)
    }
}


pub struct LogParser<'a> {
    events: Vec<Event>,
    decompressor: Decompressor<'a>,
    config: &'a ParserConfig,
    root: PathBuf
}


impl Decompress for LogParser <'_>{
    fn decompress_file(&self, filepath: PathBuf) -> Vec<String> {
        return self.decompressor.decompress(filepath);
    }
}


pub struct EventIterator <'a> {
    events: &'a Vec<Event>,
    index: usize
}


impl <'a> Iterator for EventIterator <'a> {
    type Item = &'a Event;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.events.len() {
            let res = Some(&self.events[self.index]);
            self.index += 1;
            return res;
        } else {
            None
        }
        
    }
}


impl <'a> LogParser <'a> {
    pub fn new(file_path: &str, config: &'a ParserConfig) -> LogParser<'a>{
        // check that pattern does not have errors
        return LogParser {
            events: Vec::new(),
            config: config,
            decompressor: Decompressor::new(&config.compression),
            root: PathBuf::from(file_path)
        };
    }

    pub fn iter(&self) -> EventIterator<'_> {
        EventIterator{
            events: &self.events,
            index: 0
        }
    }

    pub fn event_count(&self) -> usize {
        // this is not that useful function, but makes size calls a bit more clean
        // and removes a need to make events vector visible to scary outside world
        return self.events.len();
    }

    fn get_files(&self) -> Vec<FileEntry> {
        if !self.root.exists() {
            warn!("Provided path {} does not exist", self.root.display());
            return Vec::new();
        }

        let root = match str::strip_suffix(self.root.to_str().unwrap(), "/") {
            Some(p) => p,
            // I like life on the edge
            // this in theory can panic when pathbuf is converted into a pointer
            None => self.root.to_str().unwrap()
        };

        let pattern = format!("{}/{}", root, self.config.logfile_pattern);
        let matches: Vec<PathBuf> = glob(&pattern)
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        let mut content = Vec::new();
        for file in matches {
            content.push(FileEntry::new(file));
        }
        return content;
    }

    pub fn parse(&mut self) -> Result<(), ErrorKind> {
        let pattern = self.config.compile_message_pattern();
        if pattern.captures_len() < 3 {
            warn!("Incorrect amount of named capture groups, 3 is required {} found", pattern.captures_len());
            return Err(ErrorKind::InvalidInput);
        }

        let mut filecount: i64 = 0;
        for f in self.get_files() {
            if f.path.is_dir() {
                warn!("File {} is a directory, ignoring", f.path.display());
                continue;
            }

            filecount += 1;
            for line in self.decompress_file(f.path) {
                for entry in pattern.captures_iter(&line){
                    let message = &entry["message"];
                    if self.config.filter_event(message) {
                        continue;
                    }

                    let event = Event::new(
                        self.config.read_timestamp(&entry["timestamp"]),
                        &entry["message"],
                        &entry["loglevel"]
                    );
                    self.events.push(event);
                }
            }
        }
        info!("Parsed {} logfiles successfully", filecount);
        Ok(())
    }
}
