use std::net::SocketAddr;

pub struct Config {
    pub domain_names: Vec<String>,
    pub top_directory: String,
    pub root_directory: String,
    pub subdomain_directory: String,
    pub socket: SocketAddr,
    pub request_timeout_seconds: usize,
    pub request_initial_buffer_size_kilobytes: usize,
    pub request_maximum_buffer_size_kilobytes: usize,
    pub request_default_filename: String,
    pub not_found_filename: String,
}
