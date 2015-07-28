extern crate server;

use server::{Server, ServerInfo, run};

const CONTENT_TYPE: &'static str = "image/png";
const IMAGE_DIR: &'static str = "screenies/";

fn main() {
    run(Server::new(String::from(IMAGE_DIR), String::from(CONTENT_TYPE), 6), ServerInfo{ port: 3000, threads: 8 });
}
