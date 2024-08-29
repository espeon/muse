use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub artist_split_exceptions: Vec<String>,
}

fn create_default_config(path: &str) -> Config {
    let default_config = Config {
        artist_split_exceptions: vec![
            "ラッツ&スター".to_string(),
            // Add more default exceptions here
        ],
    };

    let config_json =
        serde_json::to_string_pretty(&default_config).expect("Failed to serialize default config");

    fs::write(path, config_json).expect("Failed to write default config to file");

    default_config
}

pub fn load_or_create_config(path: &str) -> Config {
    if Path::new(path).exists() {
        let config_file = fs::read_to_string(path).expect("Unable to read config file");
        serde_json::from_str(&config_file).expect("JSON was not well-formatted")
    } else {
        create_default_config(path)
    }
}
