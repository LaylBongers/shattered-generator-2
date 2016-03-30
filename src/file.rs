use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use encoding::{Encoding, DecoderTrap, EncoderTrap};
use encoding::all::WINDOWS_1252;

pub fn read_all_text<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();

    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    data
}

pub fn write_all_text<P: AsRef<Path>>(path: P, text: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

pub fn read_all_win_1252<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();

    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    WINDOWS_1252.decode(&data, DecoderTrap::Strict).unwrap()
}

pub fn write_all_win_1252<P: AsRef<Path>>(path: P, text: &str) {
    let mut file = File::create(path).unwrap();

    let data = WINDOWS_1252.encode(&text, EncoderTrap::Strict).unwrap();

    file.write_all(&data).unwrap();
}
