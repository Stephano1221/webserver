use std::net::SocketAddr;

use webserver::{http_parser::HttpProtocol, network, server};

fn main() {
    let http_protocol = HttpProtocol::Http;
    let port = http_protocol.port();
    let local_ipv4_address = network::get_local_ipv4_address();
    let config = server::Config {
        domain_names: vec!("example.com".to_owned(), "www.example.com".to_owned()),
        top_directory: "content".to_owned(),
        root_directory: "root".to_owned(),
        subdomain_directory: "subdomains".to_owned(),
        socket: SocketAddr::new(local_ipv4_address, port),
        request_initial_buffer_size_kilobytes: 16,
        request_maximum_buffer_size_kilobytes: 1024,
        request_default_filename: "index.html".to_owned(),
        not_found_filename: "404.html".to_owned(),
    };
    server::start_server(&config);
}
