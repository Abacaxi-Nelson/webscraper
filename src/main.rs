use std::net::TcpListener;
use std::net::TcpStream;
use std::io::{Error, Read, Write};
use std::fmt::Debug;
use std::fs;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    for stream in listener.incoming() {
        match stream {
            Err(e) => { eprintln!("error handling stream: {}", e) },
            Ok(stream) => {
                handle_connection(stream).unwrap_or_else(|error| eprintln!("Error on handle_connection {:?}", error));
            }
        }
    }
}

#[derive(Debug)]
enum HttpMethod {
    GET(String, bool),
    POST(String, bool),
    OTHER(String, bool),
}

trait Html {
    fn write(&self, stream: TcpStream) ;
}

impl Html for HttpMethod {
    fn write(&self, mut stream: TcpStream) {
        let contents = match self {
            HttpMethod::GET(path, _) => {
                println!("path: {:?}", path);

                // maybe match works better
                if path == "/" {
                    fs::read_to_string("./homepage.html").unwrap()
                }else if path == "/favicon.ico" {
                    // need to learn how to serve a favicon
                    fs::read_to_string("./homepage.html").unwrap()
                }else if path.starts_with("/search") {
                    let path = path
                        .replace("/search?", "")
                        .replace("&", " ")
                        ;
                    let url = path.split_whitespace().collect::<Vec<&str>>()[0].split("=").collect::<Vec<&str>>()[1];
                    let search = path.split_whitespace().collect::<Vec<&str>>()[1].split("=").collect::<Vec<&str>>()[1];
                    let result = search_search_in_url(url, search);

                    fs::read_to_string("./results.html").unwrap().replace("<REPLACE_ME>", search)
                }else{
                    fs::read_to_string("./void.html").unwrap()
                }
            }
            _ => fs::read_to_string("./error.html").unwrap(),
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap(); 
    }
}

fn search_search_in_url(url: String, search: String) -> String {
    let mut res = reqwest::blocking::get(url);
    let mut body = String::new();
    res.read_to_string(&mut body)?;
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Error> { 
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let content = String::from_utf8_lossy(&buffer[..]);

    let request = read_content(&content);
    println!("request: {:?}", request);
    request.write(stream);

    Ok(())
}

fn get_verb_path(method: &&str) -> HttpMethod { 
    let path = method.split_whitespace().collect::<Vec<&str>>()[1];
    match method.split_whitespace().collect::<Vec<&str>>()[0]{
        "GET"  => HttpMethod::GET(path.to_string(), false),
        "POST" => HttpMethod::POST(path.to_string(), false),
        _      => HttpMethod::OTHER(path.to_string(), false),
    }
}

fn read_content(content: &str) -> HttpMethod { 
    match content.split("\n").collect::<Vec<&str>>().as_slice().get(0) {
        Some(r) => get_verb_path(r),
        None => HttpMethod::OTHER(String::new(), false),
    }
}