use std::{fs, path::Path};

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Global {
    pub domain_names: Vec<String>,
    pub port: u16,
    pub top_directory: String,
    pub root_directory: String,
    pub subdomain_directory: String,
    pub request_initial_buffer_size_kilobytes: usize,
    pub request_maximum_buffer_size_kilobytes: usize,
    pub request_default_filename: String,
    pub not_found_filename: String,
    pub request_timeout_seconds: usize,
}

#[derive(Deserialize)]
pub struct Config {
    pub global: Global,
}

impl Config {
    /// Creates a new [`Config`] instance from a <a href="https://toml.io">TOML</a> file at `path`.
    pub fn from_file(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
        let file = fs::read_to_string(path)?;
        Self::from_str(&file)
    }

    /// Creates a new [`Config`] instance from a string containing <a href="https://toml.io">TOML</a>.
    pub fn from_str(str: &str) -> Result<Config, Box<dyn std::error::Error>> {
        match toml::from_str(str) {
            Ok(config) => Ok(config),
            Err(e) => Err(Box::new(e)),
        }
    }
}
