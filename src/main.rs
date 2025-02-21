use std::{env, path::{self, PathBuf}};

use webserver::{config::Config, server};

fn main() {
    let config_path = get_config_path();
    let config = match Config::from_toml_file(&config_path) {
        Ok(config) => config,
        Err(e) => {
            let absolute_path = match path::absolute(&config_path) {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(_) => config_path.to_string_lossy().to_string(),
            };
            let default_message = format!("Unable to read configuration file: {e}. Please ensure that a valid configuration file is found at: {absolute_path}");
            match e.downcast_ref::<std::io::Error>() { Some(err) => {
                match err.kind() {
                    std::io::ErrorKind::NotFound => {
                        eprintln!("Configuration file not found. Please ensure that a valid configuration file is found at: {}", absolute_path);
                    },
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("Permission denied reading configuration file at: {}", absolute_path);
                    },
                    _ => eprintln!("{}", default_message),
                }
            } _ => { match e.downcast_ref::<toml::de::Error>() { Some(err) => {
                eprintln!("An error occured while parsing the configuration file. Please ensure that the configuration file at {} is valid: {}", absolute_path, err);
            } _ => {
                eprintln!("{}", default_message);
            }}}}
            return
        }
    };
    server::start_server(&config);
}

fn get_config_path() -> PathBuf {
    let config_filename = "config.toml";
    let company_name = "";
    let app_name = "Webserver";

    #[cfg(target_family = "windows")]
    {
        let parent_directory = env::var("LOCALAPPDATA").expect("LOCALAPPDATA environment variable should be set");
        let config_path = format!("{parent_directory}\\{company_name}\\{app_name}\\{config_filename}");
        PathBuf::from(&config_path)
    }

    #[cfg(target_family = "unix")]
    {
        let parent_directory = "/etc";
        let config_path = format!("{parent_directory}/{}/{}/{config_filename}", company_name.to_lowercase(), app_name.to_lowercase());
        PathBuf::from(&config_path)
    }
}
