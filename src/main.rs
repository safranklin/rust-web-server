use std::fs;
use std::env;
use std::thread;
use std::process;
use std::io::prelude::*;
use std::time::Duration;
use std::net::TcpStream;
use std::net::TcpListener;

fn main() {

    // Grab the port environment variable. If it is unset default to 7878.
    let port = env::var("PORT").unwrap_or(String::from("7878"));

    // Build up the address using configurable port.
    let address = format!("127.0.0.1:{}", port);

    // Create the listener by binding to the address and port.
    let listener = TcpListener::bind(&address).unwrap_or_else(|err| {
        // If we failed to bind to the port, inform the user.
        eprintln!("Failed to bind to {}!\nSee Error:\n{}", &address, err);
        // Exit since failing to create the listener means we can't serve anything.
        process::exit(2);
    });

    // Let the user we successfully bound to the port.
    println!("Listenting on port {}...", port);

    // Iterate through each connection attempt being recieved on the listener.
    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
    
}

fn handle_connection(mut stream: TcpStream) {

    println!("Handling connection...");
    // Accept a mutable TcpStream (needs to be mutable since it keeps track (internally) 
    // of how much of the request we've read.)

    // Create a buffer big enough for handling simple requests.
    let mut buffer = [0; 1024];

    // Read the bytes off the stream buffer and store them in the buffer
    stream.read(&mut buffer).unwrap();

    // Provide a simple output of the buffer contents.
    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));


    let (status_line, filename) = if buffer.starts_with(b"GET / HTTP/1.1\r\n") {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(b"GET /sleep HTTP/1.1\r\n") {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    println!("Sent response successfully.");
}
