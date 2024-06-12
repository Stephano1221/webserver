use std::{
    error::Error, fs, io::{self, BufReader, Read, Write}, net::{IpAddr, SocketAddr, TcpListener, TcpStream}
};

use crate::http_parser::{self, HttpRequest, HttpResponse, HttpStatusCode};

pub enum HttpProtocol {
    Http,
    Https
}

impl HttpProtocol {
    pub fn get_port(&self) -> u16 {
        match self {
            HttpProtocol::Http => 80,
            HttpProtocol::Https => 443
        }
    }
}

pub fn get_local_ipv4_address() -> IpAddr {
    let get_ipv4_stream = TcpStream::connect("ipv4.icanhazip.com:443").expect("Should be able to connect to icanhazip.com");
    let local_ipv4_address = get_ipv4_stream.local_addr().expect("Should be able to read local socket address").ip();
    return local_ipv4_address;
}

pub fn start_server(socket: SocketAddr) {
    let tcp_listener = TcpListener::bind(socket).expect("Should be able to bind to local IP address");
    println!("Server started.");
    println!("Local IPv4 Address: {}", socket.ip());
    for stream in tcp_listener.incoming() {
        let stream = stream.expect("`stream` should always be `Some()`");
        accept_connection(stream);
    }
}

pub fn accept_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let stream_ip_address = stream.peer_addr().expect("`Stream` should contain the socket address of the remote peer");
    println!("Connection request from: {stream_ip_address}.");

    let mut buf_reader = BufReader::new(&mut stream);
    const BUFFER_BYTES_INITIAL_SIZE: usize = 1024 * 16;
    let mut buf = [0u8; BUFFER_BYTES_INITIAL_SIZE];
    let mut buf_received_bytes = 0;
    let mut http_request = http_parser::PartialHttpRequest::new();
    let http_request = loop {
        let bytes_read = buf_reader.read(&mut buf);
        if let Err(error) = bytes_read {
            match error.kind() {
                io::ErrorKind::Interrupted => continue,
                _ => return Err(Box::new(error)),
            }
        }
        buf_received_bytes += bytes_read.expect("`bytes_read` should always be `Ok` here");
        match http_parser::HttpRequest::try_parse(&mut http_request, &buf) {
            http_parser::Processing::InProgress(_) => continue,
            http_parser::Processing::Finished(result) => break result
        }
    };
    handle_request(&mut stream, http_request)
}

fn handle_request(stream: &mut TcpStream, http_request: Result<http_parser::HttpRequest, (io::Error, HttpStatusCode)>) -> Result<(), Box<dyn Error>> {
    match http_request {
        Err(error) => {
            let http_response = HttpResponse {
                version: http_parser::HttpVersion::Http1Dot1,
                status_code: error.1,
                header: None,
                body: None
            };
            send_response(stream, &http_response)?;
            return Err(Box::new(error.0))
        }
        Ok(request) => {
            let content_directory = "content/";
            let root_directory = "root/";
            let subdomain_directory = "sobdomains/";
            let http_response = match &request.method {
                Some(method) => match method {
                    http_parser::HttpMethod::Get => {
                        
                    },
                    http_parser::HttpMethod::Head => todo!(),
                },
                None => todo!(),
            };
                // if http_request.is_err() {
            //     let (error, status_code) = http_request.unwrap_err();
            //     let http_response = HttpResponse::new(http_parser::HttpVersion::Http1Dot1, status_code, None, None);
            //     stream.write_all(http_response.to_string().as_bytes());
            //     return Err(error)
            // }
            // let http_request = http_request.expect("`http_request` should not be `err` here");
            // let http_request = partial_http_request.request;
            // match http_request.method {
            //     Some(_) => todo!(),
            //     None => todo!(),
            // }
        
            // let contents = fs::read_to_string(format!("{content_directory}{filename}")).expect("Should be able to read requested file");
            // let length = contents.len();
        
            // let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
            // stream.write_all(response.as_bytes()).expect("Should be able to write data to the stream");
            Ok(())
        }
    }
}

fn send_response(stream: &mut TcpStream, http_response: &HttpResponse) -> Result<(), Box<dyn Error>> {
    stream.write_all(http_response.to_string().as_bytes())?;
    Ok(())
}
    
fn http_get_or_head<'a>(http_request: &'a http_parser::HttpRequest) -> HttpResponse<'a> {
    HttpResponse {
        version: http_request.version.clone().unwrap(),
        status_code: HttpStatusCode::OK200,
        header: None,
        body: None
    }
}
