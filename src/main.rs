extern crate encoding;
extern crate toml;
extern crate eu4data;

mod config;
mod file;

use std::fs;
use std::path::PathBuf;
use config::Config;
use eu4data::Eu4Table;

fn main() {
    let config = Config::load();
    prepare_output(&config);
    let source_data = load_eu4_data(&config);
    let target_data = process_eu4_data(source_data);
    write_eu4_data(&config, target_data);
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

    // Copy over the .mod file TODO: Auto-generate
    let mut shattered_mod = config.target_path.clone();
    shattered_mod.push("..");
    shattered_mod.push("shattered.mod");
    fs::copy("./assets/shattered.mod", shattered_mod).unwrap();

    println!("");
}

#[derive(Clone)]
struct FileTable {
    file_name: String,
    data: Eu4Table
}

struct Eu4SourceData {
    provinces: Vec<FileTable>,
    countries: Vec<FileTable>,
    country_history: Vec<FileTable>,
    country_tags: Vec<FileTable>,
}

fn load_eu4_data(config: &Config) -> Eu4SourceData {
    println!("=== loading eu4 game data ===");

    let provinces = load_eu4_data_from_folder(&config.game_path, "history", "provinces");
    let countries = load_eu4_data_from_folder(&config.game_path, "common", "countries");
    let country_history = load_eu4_data_from_folder(&config.game_path, "history", "countries");
    let country_tags = load_eu4_data_from_folder(&config.game_path, "common", "country_tags");

    println!("");

    Eu4SourceData {
        provinces: provinces,
        countries: countries,
        country_history: country_history,
        country_tags: country_tags,
    }
}

fn load_eu4_data_from_folder(base: &PathBuf, sub1: &str, sub2: &str) -> Vec<FileTable> {
    println!("Loading {}/{}...", sub1, sub2);

    // Get a path for the folder the data is in
    let mut dir = base.clone();
    dir.push(sub1);
    dir.push(sub2);
    assert!(dir.is_dir(), "\"{}\" is not an existing directory", dir.display());

    let mut data = Vec::new();

    // Get all the files from the directory
    for file_r in dir.read_dir().unwrap() {
        let file = file_r.unwrap();
        //println!("Loading {:?}...", file.file_name());

        // Load the file
        let text = file::read_all_win_1252(file.path());
        let file_data = Eu4Table::parse(&text);

        data.push(FileTable {
            file_name: file.file_name().to_str().unwrap().into(),
            data: file_data
        });
    }

    data
}

struct Eu4TargetData {
    provinces: Vec<FileTable>
}

fn process_eu4_data(data: Eu4SourceData) -> Eu4TargetData {
    println!("=== processing ===");

    // Create copies of data for us to use
    let mut provinces = data.provinces.clone();

    println!("Clearing history on provinces...");
    for province in &mut provinces {
        province.data.values = province.data.values.iter()
            .filter(|v| v.key.len() == 0 || !v.key.chars().nth(0).unwrap().is_digit(10))
            .map(|v| v.clone())
            .collect();
    }

    println!("");

    Eu4TargetData {
        provinces: provinces
    }
}

fn write_eu4_data(config: &Config, data: Eu4TargetData) {
    println!("=== serializing to target ===");

    println!("Serializing province files...");

    // Create the provinces directory
    let mut dir = config.target_path.clone();
    dir.push("history");
    dir.push("provinces");
    fs::create_dir_all(&dir).unwrap();

    // Write all the data to it
    for province in &data.provinces {
        let mut file = dir.clone();
        file.push(&province.file_name);

        file::write_all_win_1252(file, &province.data.serialize());
    }
}
