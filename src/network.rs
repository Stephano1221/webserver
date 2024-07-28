use std::{
    error::Error, io::{self, BufReader, Read, Write}, net::{IpAddr, TcpListener, TcpStream}, time::Instant
};

use crate::{helper::enums::Processing, http_parser::{HttpRequest, HttpStatusCode, PartialHttpRequest}, server};

pub fn start_listener(config: &server::Config) {
    let tcp_listener = TcpListener::bind(config.socket).expect("Should be able to bind to local IP address");
    println!("Server started.");
    println!("Local IPv4 Address: {}", config.socket.ip());
    for stream in tcp_listener.incoming() {
        let stream = stream.expect("`stream` should always be `Some()`");
        accept_connection(config, stream);
    }
}

pub fn get_local_ipv4_address() -> IpAddr {
    let get_ipv4_stream = TcpStream::connect("ipv4.icanhazip.com:443").expect("Should be able to connect to icanhazip.com");
    let local_ipv4_address = get_ipv4_stream.local_addr().expect("Should be able to read local socket address").ip();
    return local_ipv4_address;
}

fn accept_connection(config: &server::Config, mut stream: TcpStream) {
    let now = Instant::now();
    let stream_ip_address = stream.peer_addr().expect("`Stream` should contain the socket address of the remote peer");
    println!("Connection request from: {stream_ip_address}.");

    let mut buf_reader = BufReader::new(&mut stream);
    const BYTES_IN_KILOBYTE: usize = 1024;
    let buffer_size_bytes = BYTES_IN_KILOBYTE * config.request_initial_buffer_size_kilobytes;
    let buffer_maximum_size_bytes = BYTES_IN_KILOBYTE * config.request_maximum_buffer_size_kilobytes;
    let mut buf = vec!(0; buffer_size_bytes);
    let mut buf_received_bytes = 0;
    let mut http_request = PartialHttpRequest::new();

    let mut http_request = loop {
        if (config.request_timeout_seconds > 0) && (now.elapsed().as_secs() > 5) {
            server::handle_request(config, &mut stream, &mut Err((io::ErrorKind::Other.into(), HttpStatusCode::RequestTimeout408)));
            println!("Request from {} timed out", stream_ip_address);
            return;
        }
        let bytes_read = buf_reader.read(&mut buf);
        if let Err(error) = bytes_read {
            match error.kind() {
                io::ErrorKind::Interrupted => continue,
                _ => return,
            }
        }
        buf_received_bytes += bytes_read.expect("`bytes_read` should be `Ok` here");
        if buf_received_bytes > buffer_maximum_size_bytes {
            server::handle_request(config, &mut stream, &mut Err((io::ErrorKind::Other.into(), HttpStatusCode::ContentTooLarge413)));
            return
        }
        match HttpRequest::try_parse(&mut http_request, &buf) {
            Processing::InProgress(_) => continue,
            Processing::Finished(result) => break result
        }
    };

    server::handle_request(config, &mut stream, &mut http_request);
    println!("Handled request from {} in {}ms", stream_ip_address, now.elapsed().as_millis());
}

pub fn send_bytes(stream: &mut TcpStream, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    stream.write_all(bytes)?;
    Ok(())
}
