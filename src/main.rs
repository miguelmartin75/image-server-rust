//extern crate hyper;
extern crate server;

//use hyper::header::ContentType;

use server::Server;

const SERVER_ADDRESS: &'static str = "localhost";
const SERVER_URL: &'static str = "http://localhost";
const IMAGE_DIR: &'static str = "screenies/";
const CONTENT_TYPE: &'static str = "image/png";

//fn content_type() -> ContentType { ContentType::png() }

fn main() {
    Server::new(IMAGE_DIR, CONTENT_TYPE, SERVER_ADDRESS, SERVER_URL).run(3000, 4);

    //hyper::Server::http(my_server.run_address).unwrap().handle(my_server).unwrap();
}
