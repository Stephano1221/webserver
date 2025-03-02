use std::{
    error::Error,
    io::{self, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    config::Config,
    helper::enums::Processing,
    http_parser::{HttpRequest, HttpStatusCode, PartialHttpRequest},
    server,
};

use local_ip_address::local_ip;

pub fn start_listener(config: &Config) -> Result<(), Box<dyn Error>> {
    let local_ipv4_address = match local_ip() {
        Ok(ip_address) => ip_address,
        Err(error) => {
            eprintln!("Unable to determine local IPv4 address.");
            return Err(Box::new(error));
        }
    };
    let socket_address = SocketAddr::new(local_ipv4_address, config.global.port);
    let tcp_listener = match TcpListener::bind(socket_address) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Unable to bind to address {}.", socket_address);
            return Err(Box::new(error));
        }
    };
    println!("Server started.");
    println!("Local IPv4 Address: {}.", socket_address.ip());
    for stream in tcp_listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!(
                    "Unable to accept incoming connection with error: {}.",
                    error
                );
                continue;
            }
        };
        accept_connection(config, stream);
    }
    Ok(())
}

fn accept_connection(config: &Config, mut stream: TcpStream) {
    let now = Instant::now();
    let stream_ip_address = match stream.peer_addr() {
        Ok(socket_address) => socket_address,
        Err(error) => {
            eprintln!(
                "Unable to determine IP address of incoming connection with error: {}.",
                error
            );
            return;
        }
    };
    println!("Connection request from: {stream_ip_address}.");

    if let Err(error) = stream.set_nonblocking(true) {
        eprintln!(
            "Unable to set nonblocking. Dropping connection to prevent blocking with error: {}.",
            error
        );
        return;
    }

    let mut buf_reader = BufReader::new(&mut stream);
    const BYTES_IN_KILOBYTE: usize = 1024;
    let buffer_size_bytes = BYTES_IN_KILOBYTE * config.global.initial_buffer_size_kilobytes;
    let buffer_maximum_size_bytes = BYTES_IN_KILOBYTE * config.global.maximum_buffer_size_kilobytes;
    let mut buf = vec![0; buffer_size_bytes];
    let mut buf_received_bytes = 0;
    let mut http_request = PartialHttpRequest::new();

    let mut http_request = loop {
        if let Some(timeout_seconds) = config.global.minimum_timeout_seconds {
            if timeout_seconds > 0 && now.elapsed().as_secs() >= timeout_seconds as u64 {
                server::handle_request(
                    config,
                    &mut stream,
                    &mut Err((
                        io::ErrorKind::Other.into(),
                        HttpStatusCode::RequestTimeout408,
                    )),
                    now,
                );
                println!(
                    "Request from {} timed out after {}ms while reading.",
                    stream_ip_address,
                    now.elapsed().as_millis()
                );
                return;
            }
        }
        match buf_reader.read(&mut buf) {
            Ok(bytes) => buf_received_bytes += bytes,
            Err(error) => match error.kind() {
                io::ErrorKind::Interrupted => continue,
                io::ErrorKind::WouldBlock => {
                    sleep(Duration::from_millis(1));
                    continue;
                }
                _ => {
                    eprintln!("Error reading from stream: {}.", error);
                    return;
                }
            },
        };
        if buf_received_bytes > buffer_maximum_size_bytes {
            server::handle_request(
                config,
                &mut stream,
                &mut Err((
                    io::ErrorKind::Other.into(),
                    HttpStatusCode::ContentTooLarge413,
                )),
                now,
            );
            return;
        }
        match HttpRequest::try_parse(&mut http_request, &buf) {
            Processing::InProgress(_) => continue,
            Processing::Finished(result) => break result,
        }
    };

    server::handle_request(config, &mut stream, &mut http_request, now);
    println!(
        "Handled request from {} in {}ms.",
        stream_ip_address,
        now.elapsed().as_millis()
    );
}

pub fn send_bytes(
    config: &Config,
    stream: &mut TcpStream,
    bytes: &[u8],
    time_started_request: Instant,
) -> Result<(), Box<dyn Error>> {
    let stream_ip_address = stream.peer_addr().unwrap();
    let mut total_sent_bytes = 0;
    while total_sent_bytes < bytes.len() {
        if let Some(timeout_seconds) = config.global.minimum_timeout_seconds {
            if timeout_seconds > 0
                && time_started_request.elapsed().as_secs() >= timeout_seconds as u64
            {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!(
                        "Request from {} timed out after {}ms while writing",
                        stream_ip_address,
                        time_started_request.elapsed().as_millis()
                    ),
                )));
            }
        }
        match stream.write(&bytes[total_sent_bytes..]) {
            Ok(sent_bytes) => total_sent_bytes += sent_bytes,
            Err(error) => match error.kind() {
                io::ErrorKind::Interrupted => continue,
                io::ErrorKind::WouldBlock => {
                    sleep(Duration::from_millis(1));
                    continue;
                }
                _ => {
                    return Err(Box::new(error));
                }
            },
        };
    }
    Ok(())
}
