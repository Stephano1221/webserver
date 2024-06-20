use core::fmt;
use std::collections::{hash_map, HashMap};

use crate::helper::bytes;

#[derive(Clone, Debug)]
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
            let new_start_index = if field_name_separator_index >= unprocessed_bytes.len() { unprocessed_bytes.len() } else { field_name_separator_index + field_name_delimiter.len() };
            unprocessed_bytes = &unprocessed_bytes[new_start_index..];

            let field_value_separator_index = match bytes::find(unprocessed_bytes, line_delimiter) {
                None => unprocessed_bytes.len(),
                Some(index) => index,
            };
            let field_value = &unprocessed_bytes[..field_value_separator_index];
            let new_start_index = if field_value_separator_index >= unprocessed_bytes.len() { unprocessed_bytes.len() } else { field_value_separator_index + line_delimiter.len() };
            unprocessed_bytes = &unprocessed_bytes[new_start_index..];

            let field_name = match std::str::from_utf8(field_name) {
                Err(_) => continue,
                Ok(field_name) => field_name.trim(),
            };
            let field_value = match std::str::from_utf8(field_value) {
                Err(_) => continue,
                Ok(field_value) => field_value.trim(),
            };

            match fields.entry(field_name.to_owned()) {
                hash_map::Entry::Vacant(entry) => { entry.insert(field_value.to_owned()); },
                hash_map::Entry::Occupied(mut entry) => {
                    // let old_value = entry.get();
                    // let new_value = format!("{old_value}, {field_value}");
                    // entry.insert(new_value);

                    // let old_value = entry.get_mut();
                    // *old_value = format!("{old_value}, {field_value}");

                    *entry.get_mut() = format!("{}, {field_value}", entry.get());
                },
            };
        }
        match fields.len() {
            0 => None,
            _ => Some(HttpHeader(fields)),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_owned(), value.to_owned());
    }
}

impl fmt::Display for HttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for (key, value) in &self.0 {
            output.push_str(&format!("{key}: {value}"));
        }
        write!(f, "{output}")
    }
}
