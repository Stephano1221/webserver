use std::net::SocketAddr;

use webserver::{http_parser, network, server};

fn main() {
    let http_protocol = http_parser::HttpProtocol::Http;
    let port = http_protocol.port();
    let local_ipv4_address = network::get_local_ipv4_address();
    let config = server::Config {
        content_directory: "content".to_owned(),
        root_directory: "root".to_owned(),
        subdomain_directory: "sobdomains".to_owned(),
        socket: SocketAddr::new(local_ipv4_address, port),
        request_default_filename: "index.html".to_owned(),
        not_found_filename: "404.html".to_owned(),
    };
    network::start_listener(&config);
}
