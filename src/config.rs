use std::fs::File;
use std::io::Read;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub project: String,
    pub nodes: Vec<ConfigNode>,
    pub flows: Vec<ConfigFlow>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigNode {
    pub name: String,
    pub identifier: Option<String>,
    pub code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFlow {
    pub from: String,
    pub to: String,
}

impl Config {
    pub fn parse(file: &str) -> Config {
        let mut f = File::open(file).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the file");
        let config: Config = toml::from_str(&contents).unwrap();
        config
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use super::*;

    #[test]
    fn test_config() {
        let config: Config = Config::parse("src/fixtures/exampleProject/config.toml");
    }
}
