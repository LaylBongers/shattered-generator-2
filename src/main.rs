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

    load_eu4_data_provinces(config);

    println!("");
}

fn load_eu4_data_provinces(config: &Config) {
    println!("Loading provinces...");

    // Get a path for the folder the provinces are in
    let mut provinces_dir = config.game_path.clone();
    provinces_dir.push("history");
    provinces_dir.push("provinces");
    assert!(provinces_dir.is_dir(), "\"{}\" is not an existing directory", provinces_dir.display());

    // Get all the files from that directory
    for file_r in provinces_dir.read_dir().unwrap() {
        let file = file_r.unwrap();
        println!("Loading {:?}...", file.file_name());

        // Load the entire file
        let text = file::read_all_win_1252(file.path());
        let _data = Eu4Table::parse(&text);
    }

    println!("Done parsing provinces!");
}
