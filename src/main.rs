extern crate hyper;

extern crate server;

use hyper::header::ContentType;

use server::Server;

const SERVER_ADDRESS: &'static str = "127.0.0.1:3000";
const SERVER_URL: &'static str = "http://127.0.0.1:3000"; // TODO: change to domain
const IMAGE_DIR: &'static str = "screenies/";

fn content_type() -> ContentType { ContentType::png() }

fn main() {
    let my_server = Server::new(IMAGE_DIR, content_type(), SERVER_ADDRESS, SERVER_URL);
    hyper::Server::http(my_server.run_address).unwrap().handle(my_server).unwrap();
}
