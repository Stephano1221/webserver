use std::{
    env,
    path::{self, PathBuf},
    process::ExitCode,
};

use webserver::{config::Config, server};

fn main() -> ExitCode {
    let config_path = get_config_path();
    let config = match Config::from_toml_file(&config_path) {
        Ok(config) => config,
        Err(error) => {
            let absolute_path = match path::absolute(&config_path) {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(_) => config_path.to_string_lossy().to_string(),
            };
            let default_message = format!(
                "Unable to read configuration file: {error}. Please ensure that a valid configuration file is found at: {absolute_path}"
            );
            match error.downcast_ref::<std::io::Error>() {
                Some(err) => match err.kind() {
                    std::io::ErrorKind::NotFound => {
                        eprintln!(
                            "Configuration file not found. Please ensure that a valid configuration file is found at: {}",
                            absolute_path
                        );
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!(
                            "Permission denied reading configuration file at: {}",
                            absolute_path
                        );
                    }
                    _ => eprintln!("{}", default_message),
                },
                None => match error.downcast_ref::<toml::de::Error>() {
                    Some(err) => {
                        eprintln!(
                            "An error occured while parsing the configuration file. Please ensure that the configuration file at {} is valid: {}",
                            absolute_path, err
                        );
                    }
                    _ => {
                        eprintln!("{}", default_message);
                    }
                },
            }
            return ExitCode::FAILURE;
        }
    };
    match server::start_server(&config) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!(
                "An unrecoverable error occured. Stopping the webserver: {}",
                error
            );
            ExitCode::FAILURE
        }
    }
}

fn get_config_path() -> PathBuf {
    let config_filename = "config.toml";
    let company_name = "";
    let app_name = "Webserver";

    #[cfg(target_family = "windows")]
    {
        let parent_directory =
            env::var("LOCALAPPDATA").unwrap_or_else(|_| String::from("C:\\ProgramData"));
        let config_path =
            format!("{parent_directory}\\{company_name}\\{app_name}\\{config_filename}");
        PathBuf::from(&config_path)
    }

    #[cfg(target_family = "unix")]
    {
        let parent_directory = "/etc";
        let config_path = format!(
            "{parent_directory}/{}/{}/{config_filename}",
            company_name.to_lowercase(),
            app_name.to_lowercase()
        );
        PathBuf::from(&config_path)
    }
}
