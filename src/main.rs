use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use serde_json::json;

struct MythHandler {
    user_id: String,
    user_token: String,
}

impl MythHandler {
    fn new(user_id: String, user_token: String) -> Self {
        MythHandler { user_id, user_token }
    }

    fn handle_request(&self, stream: &mut TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer[..]);
        if request.contains("GET /launcher/GetProxyServers") {
            let response = json!({
                "code": 0,
                "msg": "success",
                "count": 1,
                "total": 1,
                "data": [{
                    "port": 25565,
                    "playerGameUuid": "1145141919810",
                    "entityId": self.user_id,
                    "token": self.user_token
                }]
            }).to_string();

            let response_headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response.len(),
                response
            );

            stream.write_all(response_headers.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}

fn start_server(user_id: String, user_token: String) {
    println!("HTTP API STARTED: http://127.0.0.1:14250/launcher/GetProxyServers");

    let listener = TcpListener::bind("127.0.0.1:14250").unwrap();
    let handler = MythHandler::new(user_id, user_token);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let handler = MythHandler::new(handler.user_id.clone(), handler.user_token.clone());
                thread::spawn(move || {
                    handler.handle_request(&mut stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn getter(addr: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    stream.write_all(&[0x01, 0x05])?;

    let mut length_buf = [0; 1];
    stream.read_exact(&mut length_buf)?;
    let length = length_buf[0] as usize;

    let mut data_buf = vec![0; length];
    stream.read_exact(&mut data_buf)?;
    let data = String::from_utf8(data_buf)?;

    let parts: Vec<&str> = data.split('|').collect();
    if parts.len() >= 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err("Invalid data format".into())
    }
}

fn main() {
    println!("EchooAPISpoofer");
    println!("Please enter server address:");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .unwrap();
    let input = input.trim();
    match getter(input) {
        Ok((user_id, user_token)) => {
            println!("EchooAPI request successful: {}|{}", user_id, user_token);
            start_server(user_id, user_token);
            println!("HTTP SERVER STOPPED");
        }
        Err(e) => {
            eprintln!("Cannot connect to Echoo API: {}", e);
            std::process::exit(114514);
        }
    }
}