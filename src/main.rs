extern crate server;

use server::Server;

const IMAGE_DIR: &'static str = "screenies/";
const CONTENT_TYPE: &'static str = "image/png";

fn main() {
    Server::new(IMAGE_DIR, CONTENT_TYPE).run(3000, 4);
}
