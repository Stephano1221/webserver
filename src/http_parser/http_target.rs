use super::HttpTargetParameters;

#[derive(Clone, Default, Debug)]
pub struct HttpTarget {
    pub path: Option<String>,
    pub parameters: Option<HttpTargetParameters>,
}

impl HttpTarget {
    /// Creates a new, empty (containing `None`) `HttpTarget`.
    pub fn new() -> Self {
        HttpTarget {
            path: None,
            parameters: None,
        }
    }

    pub fn from_str(target: &str) -> Result<Self, ()> {
        let parameter_delimiter = '?';
        // let filepath = match Filepath::from_str(target) {
        //     Err(_) => None,
        //     Ok(path) => Some(path),
        // };
        let path = match target.split_once(parameter_delimiter) {
            None => Some(target.to_owned()),
            Some((path, _)) => Some(path.to_owned()),
        };
        let parameters = match HttpTargetParameters::from_str(target) {
            Err(_) => None,
            Ok(parameters) => Some(parameters),
        };
        if !path.is_none() || !parameters.is_none() {
            Ok(HttpTarget {
                path,
                parameters,
            })
        } else {
            Err(())
        }
    }

    pub fn directory(&self) -> Option<&str> {
        self.get_directory_and_filename().0
    }

    pub fn filename(&self) -> Option<&str> {
        self.get_directory_and_filename().1
    }

    /// Sets the `path`'s filename.
    /// 
    /// # Safety
    /// The new `path` will be `Some`.
    pub fn set_filename(&mut self, filename: &str) {
        let directory_delimiter = '/';
        let old_directory = match self.directory() {
            None => Some(directory_delimiter.to_string()),
            Some(directory) => Some(directory.to_owned()),
        };
        let mut new_path = old_directory.expect("`old_directory` should be `Some`");
        if !new_path.ends_with(directory_delimiter) {
            new_path.push(directory_delimiter);
        }
        new_path.push_str(filename);
        self.path = Some(new_path);
    }

    /// Sets the `path`'s directory.
    /// 
    /// # Safety
    /// The new `path` will be `Some`.
    pub fn set_directory(&mut self, directory: &str) {
        let filename = self.filename();
        let directory_delimiter = '/';
        let mut new_path = directory.to_owned();
        if filename.is_some() {
            if !new_path.ends_with(directory_delimiter) {
                new_path.push(directory_delimiter);
            }
            new_path.push_str(filename.expect("`filename` should be `Some`"));
        }
        self.path = Some(new_path);
    }

    pub fn directory_count(&self) -> usize {
        if let None = self.path {
            return 0
        }
        let directory_delimiter = '/';
        let full_path = &self.path.as_ref().expect("`path` should be `Some`")[..];
        full_path.chars().filter(|c| *c == directory_delimiter).count()
    }

    pub fn n_directories(&self, directories: usize) -> Option<&str> {
        if let None = self.path {
            return None
        }
        let directory_delimiter = '/';
        let full_path = &self.path.as_ref().expect("`path` should be `Some`")[..];
        let mut unprocessed_path = &full_path[..];
        let mut found_directories = 0;
        let mut last_directory_separator_index = 0;
        while unprocessed_path.len() > 0 {
            match unprocessed_path.find(directory_delimiter) {
                None => return None,
                Some(index) => {
                    found_directories += 1;
                    last_directory_separator_index += index;
                    if found_directories >= directories {
                        return Some(&full_path[..=last_directory_separator_index])
                    }
                    let new_start_index = if last_directory_separator_index >= unprocessed_path.len() {
                        unprocessed_path.len()
                    } else {
                        last_directory_separator_index += 1;
                        last_directory_separator_index
                    };
                    unprocessed_path = &unprocessed_path[new_start_index..];
                },
            }
        };
        None
    }

    fn get_directory_and_filename(&self) -> (Option<&str>, Option<&str>) {
        let directory_delimiter = '/';
        let filename_extension_delimiter = '.';
        match &self.path {
            None => (None, None),
            Some(path) => {
                match path.rfind(filename_extension_delimiter) {
                    None => (Some(path), None),
                    Some(_) => {
                        match path.rfind(directory_delimiter) {
                            None => (None, Some(path)),
                            Some(index) => (Some(&path[..=index]), Some(&path[(index + 1)..])),
                        }
                    },
                }
            },
        }
    }
}
