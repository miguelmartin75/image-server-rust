extern crate server;

use server::{Server, ServerInfo, run};

const IMAGE_DIR: &'static str = "screenies/";
const CONTENT_TYPE: &'static str = "image/png";

fn main() {
    run(Server::new(String::from(IMAGE_DIR), String::from(CONTENT_TYPE)), ServerInfo{ port: 3000, threads: 8 });
}
