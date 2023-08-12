use std::path::PathBuf;
use std::str::FromStr;
use std::fs::File;
use std::io::BufReader;
use serde::{Serialize, Deserialize};
use flate2::write::{GzDecoder, ZlibDecoder};
use zip::read::ZipArchive;
use std::io::Cursor;
use std::io::prelude::*;
use std::str::from_utf8;
use log;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CompressionFormat {
    /* Make sure that as many formats are accepted as possible */
    #[serde(alias="GZIP", alias="Gzip", alias="GZip")]
    GZIP,
    #[serde(alias="TAR", alias="Tar")]
    TAR,
    #[serde(alias="ZIP", alias="Zip")]
    ZIP,
    #[serde(alias="LZ4", alias="Lz4")]
    LZ4,
    #[serde(alias="ZLIB", alias="Zlib")]
    ZLIB,
    #[serde(alias="NONE", alias="None")]
    NONE
}


impl FromStr for CompressionFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lz4" => Ok(CompressionFormat::LZ4),
            "zip" => Ok(CompressionFormat::ZIP),
            "gzip" => Ok(CompressionFormat::GZIP),
            "zlib" => Ok(CompressionFormat::ZLIB),
            "none" => Ok(CompressionFormat::NONE),
            _ => Ok(CompressionFormat::NONE),
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
    let mut writer = Vec::new();
    let mut decoder = ZlibDecoder::new(writer);
    // decode bytes into strings
    decoder.write_all(&bytes).unwrap();
    decoder.try_finish().unwrap();
    writer = match decoder.finish() {
        Ok(r) => r,
        Err(e) => panic!("Could not decompress zlib file, error {:?}", e),
    };
    return writer;
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
    /* 
    This will now break horribly when multiple files are present in an archive please fix
    Also this is slow as we need to iterate over all bytes when pushing them into the vector
    */
    let reader = Cursor::new(bytes.to_vec());
    let mut zipfile = ZipArchive::new(reader).unwrap();
    
    let mut content: Vec<u8> = Vec::new();
    for fileindex in 0..zipfile.len() {
        let mut entry = zipfile.by_index(fileindex).unwrap();
        log::debug!("Extracted {} file from zip archive", entry.name());
        // horror
        entry.read_to_end(&mut content).unwrap();
    }
    return content;
}

fn decompress_dummy(bytes: &[u8]) -> Vec<u8> {
    return bytes.to_vec();
}

impl Decompressor <'_> {
    pub fn new(algo: &CompressionFormat) -> Decompressor {
        return Decompressor{compression: algo};
    }

    pub fn decompress(&self, path: PathBuf) -> Vec<String> {
        let decompressor = match self.compression {
            CompressionFormat::LZ4 => decompress_lz4,
            CompressionFormat::ZLIB => decompress_zlib,
            CompressionFormat::TAR => decompress_gzip,
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
