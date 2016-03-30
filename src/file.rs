use std::path::Path;
use std::fs::File;
use std::io::Read;
use encoding::{Encoding, DecoderTrap};
use encoding::all::WINDOWS_1252;

pub fn read_all_text<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();

    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    data
}

pub fn read_all_win_1252<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();

    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    WINDOWS_1252.decode(&data, DecoderTrap::Strict).unwrap()
}
