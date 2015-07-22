extern crate hyper;
extern crate server;

use hyper::header::ContentType;

use server::Server;


const SERVER_ADDRESS: &'static str = "127.0.0.1:3000";
const SERVER_URL: &'static str = SERVER_ADDRESS; // TODO: change to domain
const IMAGE_DIR: &'static str = "screenies";

fn main() {
    static mut server: Server<'static> = Server{ image_dir: IMAGE_DIR, server_url: SERVER_URL, run_address: SERVER_ADDRESS };
        //Server::new(IMAGE_DIR, ContentType::png(), SERVER_URL, SERVER_ADDRESS);
    //server.run();
    //run(server);
    unsafe
    {
        let handler = server::MyHandler{ server: &mut server };
        hyper::Server::http(server.run_address).unwrap().handle(handler).unwrap();
    }
}
