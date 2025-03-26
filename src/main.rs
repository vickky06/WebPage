use dotenvy::dotenv;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, thread};
use std::{
    fs,
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    dotenv().ok();
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let addr = format!("{}:{}", host, port);

    match TcpListener::bind(&addr) {
        Ok(listener) => {
            println!("Listening on http://{}", addr);
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let mut active_requests = Arc::new(Mutex::new(0));
                        thread::spawn(move || {
                            let active_requests = Arc::clone(&active_requests);
                            {
                                // scope for the lock
                                let mut connection = active_requests.lock().unwrap();
                                *connection += 1;
                                // scope completition will automatically drop the lock
                            }
                            handle_connection(stream);
                            {
                                // scope for the lock
                                let mut connection = active_requests.lock().unwrap();
                                *connection -= 1;
                                // scope completition will automatically drop the lock
                            }
                        });
                    }
                    Err(e) => {
                        println!("Failed to handle connection: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to start server: {}", e);
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let mut request_line = buf_reader.lines().next();
    let (status_line, file_name) = match request_line.unwrap().unwrap().as_str() {
        "GET / HTTP/1.1" => (Some("HTTP/1.1 200 OK"), Some("index.html")),
        "GET /page1 HTTP/1.1" => {
            sleep(Duration::from_secs(10)); // Mimic a slow response
            (Some("HTTP/1.1 200 OK"), Some("page1.html"))
        }
        "GET /page2 HTTP/1.1" => (Some("HTTP/1.1 200 OK"), Some("page2.html")),
        _ => (Some("HTTP/1.1 404 Not Found"), Some("404.html")),
    };
    // let response = "HTTP/1.1 200 OK\r\n";
    let content = fs::read_to_string(file_name.unwrap()).unwrap();
    let content_length = content.len();
    let response = format!(
        "{}Content-Length: {}\r\n\r\n{}",
        status_line.unwrap(),
        content_length,
        content
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
