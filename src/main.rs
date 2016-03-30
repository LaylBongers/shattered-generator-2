extern crate encoding;
extern crate rand;
extern crate toml;
extern crate eu4data;

mod config;
mod file;

use std::fs;
use std::path::PathBuf;
use rand::{Rng, StdRng};
use config::Config;
use eu4data::{Eu4Table, Eu4Value};

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
    country_tags: Eu4Table,
}

fn load_eu4_data(config: &Config) -> Eu4SourceData {
    println!("=== loading eu4 game data ===");

    let provinces = load_eu4_data_from_folder(&config.game_path, "history", "provinces");
    let countries = load_eu4_data_from_folder(&config.game_path, "common", "countries");
    let country_history = load_eu4_data_from_folder(&config.game_path, "history", "countries");

    println!("Loading country tags...");
    let mut file = config.game_path.clone();
    file.push("common"); file.push("country_tags"); file.push("00_countries.txt");
    let text = file::read_all_win_1252(file);
    let country_tags = Eu4Table::parse(&text);

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
    provinces: Vec<FileTable>,
    countries: Vec<FileTable>,
    country_history: Vec<FileTable>,
    country_tags: Eu4Table
}

fn process_eu4_data(data: Eu4SourceData) -> Eu4TargetData {
    println!("=== processing ===");

    // Create copies of data for us to use
    let mut provinces = data.provinces.clone();
    let mut countries: Vec<FileTable> = Vec::new();
    let mut country_history: Vec<FileTable> = Vec::new();
    let mut country_tags = data.country_tags.clone();

    println!("Clearing events on provinces...");
    for province in &mut provinces {
        clear_events(&mut province.data);
    }

    println!("Generating new countries...");
    let mut rand = StdRng::new().unwrap();
    let mut tag_num = 0;
    for province in &mut provinces {
        // Check the province's owner tag, if it has one
        let old_country_tag = {
            if let Some(tag) = province.data.get("owner") {
                tag.as_str().to_string()
            } else {
                continue; // No owner, we can skip this province
            }
        };

        // Find the country data for this province
        let old_country_file = country_tags.get(&old_country_tag).unwrap().as_str().to_string();
        let old_country = data.countries.iter()
            .find(|d| (String::from("countries/") + &d.file_name) == old_country_file).unwrap();
        let old_country_history = data.country_history.iter()
            .find(|f| f.file_name.starts_with(&old_country_tag)).unwrap();

        // Find out the name of this province from the file
        // TODO: Retrieve this from the localization file instead of from the file name
        let province_segments: Vec<_> = province.file_name
            .split(|c| c == ' ' || c == '-' || c == '.')
            .filter(|s| s.len() != 0)
            .collect();
        let province_name = province_segments.iter().nth(1).unwrap().to_string();
        let province_id = province_segments.iter().nth(0).unwrap().to_string();
        let new_country_file_name = format!("{}.txt", province_name);

        // Make a new country with the old country's and data
        let mut new_country = old_country.clone();
        new_country.file_name = new_country_file_name.clone();
        let mut new_country_history = old_country_history.clone();

        // Clear the events on the new country
        clear_events(&mut new_country.data);
        clear_events(&mut new_country_history.data);

        // Generate a new tag for the country and add it to the tags list
        let new_country_tag = get_next_valid_tag(&mut tag_num, &country_tags);
        country_tags.set(
            &new_country_tag,
            Eu4Value::String(String::from("countries/") + &new_country_file_name));
        new_country_history.file_name = format!("{} - {}", new_country_tag, new_country_file_name);

        // Make the country's culture and religion match the province it was generated from
        new_country_history.data.set("culture", province.data.get("culture").unwrap().clone());
        new_country_history.data.set("religion", province.data.get("religion").unwrap().clone());

        // Generate a color for the country
        // TODO: Improve color generation
        // TODO: Generate flags
        new_country.data.set("color", Eu4Value::color(rand.gen(), rand.gen(), rand.gen()));

        // Update the province to be owned by the new country
        province.data.set("owner", Eu4Value::String(new_country_tag.clone()));
        province.data.set("controller", Eu4Value::String(new_country_tag.clone()));
        province.data.set("add_core", Eu4Value::String(new_country_tag.clone()));

        // Fix the HRE electors, only stay an elector if the country was the old country's capital
        if new_country_history.data.get("elector").map(|v| v.as_str() == "yes").unwrap_or(false) {
            if old_country_history.data.get("capital").unwrap().as_str() != province_id {
                new_country_history.data.set("elector", Eu4Value::String("no".into()));
            } else {
                println!("Granted elector status to {}", province_name);
            }
        }

        // Store the actual data in the lists
        countries.push(new_country);
        country_history.push(new_country_history);
    }

    println!("");

    Eu4TargetData {
        provinces: provinces,
        countries: countries,
        country_history: country_history,
        country_tags: country_tags,
    }
}

fn clear_events(table: &mut Eu4Table) {
    table.values = table.values.iter()
        .filter(|v| v.key.len() == 0 || !v.key.chars().nth(0).unwrap().is_digit(10))
        .map(|v| v.clone())
        .collect();
}

fn get_next_valid_tag(tag_num: &mut i32, country_tags: &Eu4Table) -> String {
    loop {
        // Get the next tag and increment
        let tag = get_tag_for_num(*tag_num);
        *tag_num += 1;

        // Make sure it's not one of these special cases
        if tag == "AUX" || tag == "CON" || tag == "AND" {
            continue;
        }

        // Make sure it's not already in use
        if country_tags.values.iter().any(|v| v.key == tag) {
            continue;
        }

        // It's valid, return it
        return tag;
    }
}

fn get_tag_for_num(num: i32) -> String {
    let mut b = [b'A'; 3];

    b[0] += (num / (26*26)) as u8;
    b[1] += ((num % (26*26)) / 26) as u8;
    b[2] += (num % 26) as u8;

    ::std::str::from_utf8(&b).unwrap().to_string()
}

fn write_eu4_data(config: &Config, data: Eu4TargetData) {
    println!("=== serializing to target ===");

    write_eu4_data_to_folder(&config.target_path, "history", "provinces", &data.provinces);
    write_eu4_data_to_folder(&config.target_path, "common", "countries", &data.countries);
    write_eu4_data_to_folder(&config.target_path, "history", "countries", &data.country_history);

    // Create the country tags file
    println!("Serializing country tags...");
    let mut file = config.target_path.clone();
    file.push("common"); file.push("country_tags");
    fs::create_dir_all(&file).unwrap();
    file.push("00_countries.txt");
    file::write_all_win_1252(file, &data.country_tags.serialize());
}

fn write_eu4_data_to_folder(base: &PathBuf, sub1: &str, sub2: &str, entries: &Vec<FileTable>) {
    println!("Serializing {}/{}...", sub1, sub2);

    // Create the directory
    let mut dir = base.clone();
    dir.push(sub1);
    dir.push(sub2);
    fs::create_dir_all(&dir).unwrap();

    // Write all the data to it
    for entry in entries {
        let mut file = dir.clone();
        file.push(&entry.file_name);

        file::write_all_win_1252(file, &entry.data.serialize());
    }
}
