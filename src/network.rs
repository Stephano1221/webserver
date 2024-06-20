use std::{
    error::Error, io::{self, BufReader, Read, Write}, net::{IpAddr, TcpListener, TcpStream}
};

use crate::{helper::enums::Processing, http_parser::{HttpRequest, HttpResponse, HttpStatusCode, PartialHttpRequest}, server};

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
        let bytes_read = buf_reader.read(&mut buf);
        if let Err(error) = bytes_read {
            match error.kind() {
                io::ErrorKind::Interrupted => continue,
                _ => return,
            }
        }
        buf_received_bytes += bytes_read.expect("`bytes_read` should be `Ok` here");
        if buf_received_bytes > buffer_maximum_size_bytes {
            handle_request(config, &mut stream, &mut Err((io::ErrorKind::Other.into(), HttpStatusCode::ContentTooLarge413)));
            return
        }
        match HttpRequest::try_parse(&mut http_request, &buf) {
            Processing::InProgress(_) => continue,
            Processing::Finished(result) => break result
        }
    };
    
    handle_request(config, &mut stream, &mut http_request);
}

fn handle_request(config: &server::Config, stream: &mut TcpStream, http_request: &mut Result<HttpRequest, (io::Error, HttpStatusCode)>) {
    let (http_response) = server::get_response(config, http_request);
    match &http_response {
        None => return,
        Some(response) => {
            match send_response(stream, &response) {
                Err(_error) => todo!(),
                Ok(()) => return,
            }
        },
    }
}

fn send_response(stream: &mut TcpStream, http_response: &HttpResponse) -> Result<(), Box<dyn Error>> {
    println!("Response: {}", http_response.to_string());
    stream.write_all(&http_response.as_bytes())?;
    Ok(())
}

fn log_error(config: &server::Config, http_request: &Result<HttpRequest, (io::Error, HttpStatusCode)>, response: &Option<HttpResponse>, error: &Option<&Box<dyn Error>>) {
    todo!();
}