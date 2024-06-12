use std::net::SocketAddr;

fn main() {
    let http_protocol = webserver::network::HttpProtocol::Http;
    let port = http_protocol.get_port();
    let local_ipv4_address = webserver::network::get_local_ipv4_address();
    webserver::network::start_server(SocketAddr::new(local_ipv4_address, port));
}
