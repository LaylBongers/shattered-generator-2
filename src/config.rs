use std::path::PathBuf;
use toml::Parser;
use file;

pub struct Config {
    pub mod_name: PathBuf,
    pub target_path: PathBuf,
    pub game_path: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        println!("=== loading config ===");
        println!("Loading config at \"./config/Config.toml\"...");
        let toml = file::read_all_text("./config/Config.toml");

        println!("Parsing config file...");
        let values = Parser::new(&toml).parse().unwrap();

        let config = Config {
            mod_name: values["mod_name"].as_str().unwrap().into(),
            target_path: values["target_path"].as_str().unwrap().into(),
            game_path: values["game_path"].as_str().unwrap().into(),
        };

        println!("");
        config
    }
}
