extern crate combine;
extern crate encoding;
extern crate toml;

mod config;
mod eu4data;
mod file;

use std::fs;
use config::Config;
use eu4data::Eu4Table;

fn main() {
    let config = Config::load();
    prepare_output(&config);
    load_eu4_data(&config);
}

fn prepare_output(config: &Config) {
    println!("=== preparing output ===");
    println!("Preparing mod folder at \"{}\"...", config.target_path.display());

    // Delete the old mod folder if it's already there
    if config.target_path.is_dir() {
        println!("Target already exists, deleting stale...");
        fs::remove_dir_all(&config.target_path).unwrap();
    }

    // Create a new mod folder for us
    fs::create_dir_all(&config.target_path).unwrap();

    println!("");
}

fn load_eu4_data(config: &Config) {
    println!("=== loading eu4 game data ===");

    let provinces = load_eu4_data_provinces(config);
    let countries = load_eu4_data_countries(config);

    println!("");
}

fn load_eu4_data_provinces(config: &Config) -> Vec<Eu4Table> {
    println!("Loading provinces...");

    // Get a path for the folder the provinces are in
    let mut dir = config.game_path.clone();
    dir.push("history");
    dir.push("provinces");
    assert!(dir.is_dir(), "\"{}\" is not an existing directory", dir.display());

    // Get all the files from that directory
    let mut provinces = Vec::new();
    for file_r in dir.read_dir().unwrap() {
        let file = file_r.unwrap();
        //println!("Loading {:?}...", file.file_name());

        // Load the entire file
        let text = file::read_all_win_1252(file.path());
        provinces.push(Eu4Table::parse(&text));
    }

    provinces
}

fn load_eu4_data_countries(config: &Config) -> Vec<Eu4Table> {
    println!("Loading countries...");

    // Get a path for the folder the provinces are in
    let mut dir = config.game_path.clone();
    dir.push("common");
    dir.push("countries");
    assert!(dir.is_dir(), "\"{}\" is not an existing directory", dir.display());

    // Get all the files from that directory
    let mut countries = Vec::new();
    for file_r in dir.read_dir().unwrap() {
        let file = file_r.unwrap();
        //println!("Loading {:?}...", file.file_name());

        // Load the entire file
        let text = file::read_all_win_1252(file.path());
        countries.push(Eu4Table::parse(&text));
    }

    countries
}
