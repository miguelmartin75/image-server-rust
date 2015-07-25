extern crate server;

use server::Server;

const URL: &'static str = "example.com";
const IMAGE_DIR: &'static str = "screenies/";
const CONTENT_TYPE: &'static str = "image/png";

fn main() {
    Server::new(IMAGE_DIR, CONTENT_TYPE, URL).run(3000, 4);
}
