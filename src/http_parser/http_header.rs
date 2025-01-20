use core::fmt;
use std::collections::{hash_map, HashMap};

use crate::helper::bytes;

#[derive(Clone, Debug, PartialEq)]
pub struct HttpHeader(pub HashMap<String, String>);

impl HttpHeader {
    pub fn new() -> Self {
        HttpHeader(HashMap::new())
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut fields = HashMap::new();
        let field_name_delimiter = b":";
        let line_delimiter = b"\r\n";
        let mut unprocessed_bytes = &bytes[..];
        while unprocessed_bytes.len() > 0 {
            let field_name_separator_index = match bytes::find(unprocessed_bytes, field_name_delimiter) {
                None => break,
                Some(index) => index,
            };

            let field_name = &unprocessed_bytes[..field_name_separator_index];
            let field_name = match std::str::from_utf8(field_name) {
                Err(_) => return None,
                Ok(field_name) => field_name,
            };
            if field_name.len() == 0 || field_name.contains(|c: char| c.is_whitespace()) {
                return None
            }

            let new_start_index = if field_name_separator_index >= unprocessed_bytes.len() { unprocessed_bytes.len() } else { field_name_separator_index + field_name_delimiter.len() };
            unprocessed_bytes = &unprocessed_bytes[new_start_index..];

            let field_value_separator_index = match bytes::find(unprocessed_bytes, line_delimiter) {
                None => unprocessed_bytes.len(),
                Some(index) => index,
            };

            let field_value = &unprocessed_bytes[..field_value_separator_index];
            let field_value = match std::str::from_utf8(&field_value) {
                Err(_) => return None,
                Ok(field_value) => field_value.trim(),
            };

            let new_start_index = if field_value_separator_index >= unprocessed_bytes.len() { unprocessed_bytes.len() } else { field_value_separator_index + line_delimiter.len() };
            unprocessed_bytes = &unprocessed_bytes[new_start_index..];

            let field_value = field_value.replace(|c: char| ['\r', '\n'].contains(&c), " ");

            match fields.entry(field_name.to_owned()) {
                hash_map::Entry::Vacant(entry) => { entry.insert(field_value.to_owned()); },
                hash_map::Entry::Occupied(mut entry) => { *entry.get_mut() = format!("{}, {field_value}", entry.get()); },
            };
        }
        match fields.len() {
            0 => None,
            _ => Some(HttpHeader(fields)),
        }
    }

    pub fn get_value(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_owned(), value.to_owned());
    }
}

impl fmt::Display for HttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for (key, value) in &self.0 {
            output.push_str(&format!("{key}: {value}\n"));
        }
        let _ = output.split_off(output.len() - 1);
        write!(f, "{output}")
    }
}
