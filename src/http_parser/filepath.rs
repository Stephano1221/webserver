use core::fmt;

#[derive(Clone)]
pub struct Filepath {
    pub directory: Option<String>,
    pub filename: Option<String>,
}

impl Filepath {
    pub fn empty() -> Self {
        Filepath {
            directory: None,
            filename: None,
        }
    }

    pub fn from_str(slice: &str) -> Result<Self, ()> {
        let directory_delimiter = '/';
        let file_extension_delimiter = '.';
        match slice.rfind(directory_delimiter) {
            None => Err(()),
            Some(directory_index) => {
                if directory_index == slice.len() - 1 {
                    return Ok(Filepath {
                        directory: Some(slice.to_owned()),
                        filename: None,
                    })
                }
                match slice[(directory_index + 1)..].find(file_extension_delimiter) {
                    None => Ok(Filepath {
                        directory: Some(slice.to_owned()),
                        filename: None,
                    }),
                    Some(_file_extension_index) => Ok(Filepath {
                        directory: Some(slice[..=directory_index].to_owned()),
                        filename: Some(slice[(directory_index + 1)..].to_owned()),
                    }),
                }
            },
        }
    }
}

impl fmt::Display for Filepath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let directory = match &self.directory{
            None => "",
            Some(directory) => directory,
        };
        let filename = match &self.filename{
            None => "",
            Some(filename) => filename,
        };
        write!(f, "{}{}", directory, filename)
    }
}
