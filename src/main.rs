use std::net::TcpListener;
use std::process;
use std::env;

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
    for _ in listener.incoming() {
        println!("Connection established!");
    }
    
}
