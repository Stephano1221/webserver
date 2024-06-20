use std::collections::{hash_map, HashMap};

#[derive(Clone, Debug)]
pub struct HttpTargetParameters(HashMap<String, Vec<String>>);

impl HttpTargetParameters {
    pub fn from_str(target: &str) -> Result<Self, ()> {
        let mut parameters = HashMap::new();
        let query_delimiter = '?';
        let target = match target.find(query_delimiter) {
            None => &target[..],
            Some(index) => &target[(index + 1)..],
        };
        let key_delimiter = '=';
        let parameter_delimiter = '&';
        let mut unprocessed_text = &target[..];
        while unprocessed_text.len() > 0 {
            let key_separator_index = match unprocessed_text.find(key_delimiter) {
                None => break,
                Some(index) => index,
            };
            let key = &unprocessed_text[..key_separator_index];
            let new_start_index = if key_separator_index >= unprocessed_text.len() { unprocessed_text.len() } else { key_separator_index + 1 };
            unprocessed_text = &unprocessed_text[new_start_index..];

            let parameter_separator_index = match unprocessed_text.find(parameter_delimiter) {
                None => unprocessed_text.len(),
                Some(index) => index,
            };
            let value = &unprocessed_text[..parameter_separator_index];
            let new_start_index = if parameter_separator_index >= unprocessed_text.len() { unprocessed_text.len() } else { parameter_separator_index + 1 };
            unprocessed_text = &unprocessed_text[new_start_index..];

            match parameters.entry(key.to_owned()) {
                hash_map::Entry::Vacant(entry) => {
                    let mut values = Vec::new();
                    values.push(value.to_owned());
                    entry.insert(values);
                },
                hash_map::Entry::Occupied(mut entry) => {
                    let values = entry.get_mut();
                    values.push(value.to_owned());
                },
            }
        }
        match parameters.len() {
            0 => Err(()),
            _ => Ok(Self(parameters)),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.0.get(key)
    }
}
