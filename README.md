# rust-web-server

My implementation of the 'Building a Multithreaded Web Server' final project from 
[Chapter 20](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html) 
of the [Rust Book](https://doc.rust-lang.org/book/).


# Running

`cargo run`

Once the server is started you can send requests like so:

Requests at `/` (root) should load the contents of [hello.html](hello.html).

Example: `curl http://localhost:7878/`


Requests at any other route, like `/foo` should load the contents of [404.html](404.html).

Example: `curl http://localhost:7878/foo`

You can test the multithreading capabilities by using the sleep route `/sleep` and then
trying to load the root.

The first request should wait 5 seconds before generating a response and the second
request should respond immediately. 

Example: `curl http://localhost:7878/sleep`

Example: `curl http://localhost:7878/`