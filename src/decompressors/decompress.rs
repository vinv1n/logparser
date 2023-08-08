use std::path::PathBuf;
use std::str::FromStr;
use std::fs::File;
use std::io::BufReader;
use serde::{Serialize, Deserialize};
use flate2::write::GzDecoder;
use std::io::prelude::*;
use std::str::from_utf8;
use log;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum CompressionFormat {
    GZIP,
    TAR,
    ZIP,
    LZ4,
    ZLIB,
    None
}

impl FromStr for CompressionFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lz4" => Ok(CompressionFormat::LZ4),
            "zip" => Ok(CompressionFormat::ZIP),
            "gzip" => Ok(CompressionFormat::GZIP),
            "zlib" => Ok(CompressionFormat::ZLIB),
            "none" => Ok(CompressionFormat::None),
            _ => Ok(CompressionFormat::None),
        }
    }
}

/* Log file decompressor */
#[derive(Debug)]
pub struct Decompressor <'a> {
    compression: &'a CompressionFormat
}

/* Decompression functions */
fn decompress_lz4(bytes: &[u8]) -> Vec<u8> {
    return bytes.to_vec();
}

fn decompress_zlib(bytes: &[u8]) -> Vec<u8>{
    return bytes.to_vec();
}

fn decompress_gzip(bytes: &[u8]) -> Vec<u8> {
    let mut writer = Vec::new();
    let mut decoder = GzDecoder::new(writer);
    // decode bytes into strings
    decoder.write_all(&bytes).unwrap();
    decoder.try_finish().unwrap();
    writer = match decoder.finish() {
        Ok(r) => r,
        Err(e) => panic!("Could not decompress gzip file, error {:?}", e),
    };
    return writer;
}

fn decompress_zip(bytes: &[u8]) -> Vec<u8> {
    return bytes.to_vec();
}

fn decompress_tar(bytes: &[u8]) -> Vec<u8> {
    return bytes.to_vec();
}

fn decompress_dummy(bytes: &[u8]) -> Vec<u8> {
    return bytes.to_vec();
}

impl Decompressor <'_> {
    pub fn new(algo: &CompressionFormat) -> Decompressor {
        return Decompressor{compression: algo};
    }

    pub fn decompress(&self, path: PathBuf) -> Vec<String>{
        let decompressor = match self.compression {
            CompressionFormat::LZ4 => decompress_lz4,
            CompressionFormat::TAR => decompress_tar,
            CompressionFormat::ZLIB => decompress_zlib,
            CompressionFormat::GZIP => decompress_gzip,
            CompressionFormat::ZIP => decompress_zip,
            _ => decompress_dummy,
        };

        log::info!("Decompressing file {:?}, using {:?}", path.display(), self.compression);

        // read file content
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        let mut bytes = Vec::new();

        // maybe add error handling here
        reader.read_to_end(&mut bytes).unwrap();

        let content: Vec<u8> = decompressor(bytes.as_slice());
        let s = match from_utf8(&content) {
            Ok(v) => v,
            Err(e) => panic!("Invalid utf-8 string sequnce {:?}", e)
        };
        let lines = s.split("\n").map(|l| l.to_owned()).collect();
        return lines;
    }
}

// interface for decompression
pub trait Decompress {
    fn decompress_file(&self, filepath: PathBuf) -> Vec<String>;
}
