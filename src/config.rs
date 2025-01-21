use std::{fs, path::Path};

use toml::{Table, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub global: Global,
}

impl Config {
    /// Creates a new [Config] with default values.
    pub fn new() -> Self {
        Config::default()
    }

    /// Creates a new [`Config`] instance from a <a href="https://toml.io">TOML</a> file at `path`.
    /// 
    /// For configuration values that are not specified, default values are used instead.
    pub fn from_toml_file(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
        let file = fs::read_to_string(path)?;
        Self::from_toml_str(&file)
    }

    /// Creates a new [`Config`] instance from a string containing <a href="https://toml.io">TOML</a>.
    /// 
    /// For configuration values that are not specified, default values are used instead.
    pub fn from_toml_str(str: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let table = Some(str.parse::<Table>()?);

        let global_default = Global::default();
        let global = Self::get_option_table("global", &table, &None);

        let global_primary_domain_names = Self::get_option_vec_string("primary_domain_names", &global, &global_default.primary_domain_names);
        let global_port = Self::get_u16("port", &global, &global_default.port);
        let global_parent_directory = Self::get_string("parent_directory", &global, &global_default.parent_directory);
        let global_primary_domain_folder_name = Self::get_string("primary_domain_folder_name", &global, &global_default.primary_domain_folder_name);
        let global_subdomains_folder_name = Self::get_string("subdomains_folder_name", &global, &global_default.subdomains_folder_name);
        let global_default_filename = Self::get_string("default_filename", &global, &global_default.default_filename);
        let global_not_found_filename = Self::get_string("not_found_filename", &global, &global_default.not_found_filename);
        let global_minimum_timeout_seconds = Self::get_option_usize("minimum_timeout_seconds", &global, &global_default.minimum_timeout_seconds);;
        let global_initial_buffer_size_kilobytes = Self::get_usize("initial_buffer_size_kilobytes", &global, &global_default.initial_buffer_size_kilobytes);
        let global_maximum_buffer_size_kilobytes = Self::get_usize("maximum_buffer_size_kilobytes", &global, &global_default.maximum_buffer_size_kilobytes);

        Ok(Self {
            global: Global {
                primary_domain_names: global_primary_domain_names,
                port: global_port,
                parent_directory: global_parent_directory,
                primary_domain_folder_name: global_primary_domain_folder_name,
                subdomains_folder_name: global_subdomains_folder_name,
                default_filename: global_default_filename,
                not_found_filename: global_not_found_filename,
                minimum_timeout_seconds: global_minimum_timeout_seconds,
                initial_buffer_size_kilobytes: global_initial_buffer_size_kilobytes,
                maximum_buffer_size_kilobytes: global_maximum_buffer_size_kilobytes,
            }
        })
    }

    fn get_value_as_option<'a>(key: &str, table: &'a Option<Table>) -> Option<&'a Value> {
        table.as_ref()
            .and_then(|map| map.get(key))
    }

    fn get_option_table(key: &str, table: &Option<Table>, default: &Option<Table>) -> Option<Table> {
        Self::get_value_as_option(key, table)
            .and_then(|value| value.as_table())
            .map(|value| value.clone())
            .or(default.clone())
    }

    fn get_option_vec_string(key: &str, table: &Option<Table>, default: &Option<Vec<String>>) -> Option<Vec<String>> {
        Self::get_value_as_option(key, table)
            .and_then(|value| value.as_array())
            .map(|vec_of_value| vec_of_value.iter().filter_map(|value| value.as_str().map(String::from)).collect())
            .or(default.clone())
    }

    fn get_u16(key: &str, table: &Option<Table>, default: &u16) -> u16 {
        Self::get_value_as_option(key, table)
            .and_then(|value| value.as_integer())
            .map(|value| value as u16)
            .unwrap_or(default.clone())
    }

    fn get_usize(key: &str, table: &Option<Table>, default: &usize) -> usize {
        Self::get_value_as_option(key, table)
            .and_then(|value| value.as_integer())
            .map(|value| value as usize)
            .unwrap_or(default.clone())
    }

    fn get_option_usize(key: &str, table: &Option<Table>, default: &Option<usize>) -> Option<usize> {
        Self::get_value_as_option(key, table)
            .and_then(|value| value.as_integer())
            .map(|value| value as usize)
            .or(default.clone())
    }

    fn get_string(key: &str, table: &Option<Table>, default: &str) -> String {
        Self::get_value_as_option(key, table)
            .and_then(|value| value.as_str())
            .map(|value| value.to_owned())
            .unwrap_or(default.to_owned())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            global: Global::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Global {
    pub primary_domain_names: Option<Vec<String>>,
    pub port: u16,
    pub parent_directory: String,
    pub primary_domain_folder_name: String,
    pub subdomains_folder_name: String,
    pub default_filename: String,
    pub not_found_filename: String,
    pub minimum_timeout_seconds: Option<usize>,
    pub initial_buffer_size_kilobytes: usize,
    pub maximum_buffer_size_kilobytes: usize,
}

impl Global {
    pub fn new() -> Self {
        Global::default()
    }
}

impl Default for Global {
    fn default() -> Self {
        Global {
            primary_domain_names: None,
            port: 80,
            parent_directory: String::from("content"),
            primary_domain_folder_name: String::from("primary_domain"),
            subdomains_folder_name: String::from("subdomains"),
            default_filename: String::from("index.html"),
            not_found_filename: String::from("404.html"),
            minimum_timeout_seconds: Some(5),
            initial_buffer_size_kilobytes: 16,
            maximum_buffer_size_kilobytes: 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    mod config {
        use super::super::*;

        #[test]
        fn from_valid_str() {
            let toml = r#"
                [global]
                primary_domain_names = ["example.com", "www.example.com"]
                port = 80
                parent_directory = "content"
                primary_domain_folder_name = "root"
                subdomains_folder_name = "subdomains"
                default_filename = "index.html"
                not_found_filename = "404.html"
                minimum_timeout_seconds = 5
                initial_buffer_size_kilobytes = 16
                maximum_buffer_size_kilobytes = 1024
                "#;
            
            let result = Config::from_toml_str(toml).unwrap();
            let expected_result = Config {
                global: Global {
                    primary_domain_names: Some(vec![String::from("example.com"), String::from("www.example.com")]),
                    port: 80,
                    parent_directory: String::from("content"),
                    primary_domain_folder_name: String::from("root"),
                    subdomains_folder_name: String::from("subdomains"),
                    default_filename: String::from("index.html"),
                    not_found_filename: String::from("404.html"),
                    minimum_timeout_seconds: Some(5),
                    initial_buffer_size_kilobytes: 16,
                    maximum_buffer_size_kilobytes: 1024,
                }
            };

            assert_eq!(result, expected_result);
        }

        #[test]
        fn from_string_without_global_header() {
            let toml = r#"
                domain_names = ["example.com", "www.example.com"]
                port = 80
                top_directory = "content"
                root_directory = "root"
                subdomain_directory = "subdomains"
                request_initial_buffer_size_kilobytes = 16
                request_maximum_buffer_size_kilobytes = 1024
                default_filename = "index.html"
                not_found_filename = "404.html"
                request_timeout_seconds = 5
                "#;
            
            let result = Config::from_toml_str(toml).unwrap();
            let expected_result = Config::default();

            assert_eq!(result, expected_result);
        }

        #[test]
        fn from_empty_string() {
            let toml = "";
            
            let result = Config::from_toml_str(toml).unwrap();
            let expected_result = Config::default();

            assert_eq!(result, expected_result);
        }
    }
}
