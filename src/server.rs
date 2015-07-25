extern crate tiny_http;
extern crate ascii;
extern crate rand;

use std::io::Write;
use std::fs;
use std::fs::File;
use std::borrow::Borrow;
use std::str::FromStr;

use std::sync::Mutex;

use tiny_http::{ServerBuilder, Response, Request, Header, StatusCode};

use util::read_whole;

pub use image_url::*;

macro_rules! try_print(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { println!("Error: {}", e); return; }
        }
    }}
);

pub struct Server<'a> {
    pub image_dir : &'a str,
    pub content_type: ContentType<'a>,
    //pub server_url: &'a str,
    pub used_urls: Mutex<UsedUrlSet>,
}

pub type ContentType<'a> = &'a str;

impl <'a> Server<'a> {

    pub fn new(image_dir: &'a str, content_type: ContentType<'a>) -> Server<'a> {
        let used_urls = match fs::read_dir(image_dir) {
            Ok(paths) => {
                let mut urls = UsedUrlSet::new();
                for path in paths {
                    match path.unwrap().file_name().into_string() {
                        Ok(s) => {
                            match ImageUrl::from_str(s.split(".").nth(0).unwrap()) {
                                Ok(num) => { urls.insert(num); }
                                Err(_) => { continue; }
                            }
                        },
                        Err(_) => { continue; }
                    }
                }

                urls
            }
            Err(_) => UsedUrlSet::new()
        };

        for i in used_urls.iter() {
            println!("url exists: {}", i);
        }

        Server { image_dir: image_dir, 
                 content_type: content_type,
                 server_url: server_url, 
                 used_urls: Mutex::new(used_urls),
                 }
    }

    pub fn run(&self, port: u16, threads: u32) {
        let server = ServerBuilder::new().with_port(port).build().unwrap();

        // TODO: multiple worker threads
        loop {
            match server.recv() {
                Ok(req) => {
                    self.on_req(req);
                },
                Err(e) => { println!("Error: {}", e); continue; }
            }
        }

        /*
        let guards = Vec::with_capacity(threads);

        for i in 0..threads {
            let server = tiny_http_server.clone();

            let guard = thread::spawn(move || {
                loop { 
                    match server.recv() {
                        Ok(req) => {
                            self.on_req(req);
                        },
                        Err(e) => { println!("Error: {}", e); continue; }
                    }
                }
            });

            guards.push(guard);
        }
        */
    }

    // retrieves an image path
    // name.png => <image_dir>/name
    // name => <image_dir>/name
    fn get_image_path(&self, path: &str) -> String {
        // remove extension
        let base = path.split(".").nth(0).unwrap();
        self.image_dir.to_string() + base
    }

    fn handle_get_image_req(&self, req: Request) {
        let full_path = self.get_image_path(req.url().borrow());
        match fs::metadata(&full_path) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    let res = Response::from_string("404").with_status_code(StatusCode(404));
                    req.respond(res);
                    return;
                }

                match File::open(&full_path) {
                    Ok(file) => {
                        let res = Response::from_file(file)
                                  .with_header(Header::from_bytes(&b"Content-Type"[..], self.content_type.as_bytes()).unwrap())
                                  .with_status_code(StatusCode(200));
                        req.respond(res);
                    },
                    Err(_) => {
                        let res = Response::from_string("500").with_status_code(StatusCode(500));
                        req.respond(res);
                    }
                }
            }, 
            Err(_) => {
                let res = Response::from_string("404").with_status_code(StatusCode(404));
                req.respond(res);
            }
        }
    }

    fn upload_image(&self, data: &[u8]) -> Option<String> {

        let mut used_urls = self.used_urls.lock().unwrap();

        let mut num = gen_image_url();
        while used_urls.contains(&num) {
            num = gen_image_url();
        }

        let file_name = self.get_image_path(num.to_string().borrow());
        used_urls.insert(ImageUrl(num.0));

        return match File::create(&file_name) {
            Ok(mut file) => {
                match file.write_all(data) {
                    Ok(_) => Some(num.to_string().borrow()),
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    }

    fn on_req(&self, mut req: Request) {
        // determine what request they are making:
        // if it is a GET method, then retrieve the image
        // if it is a POST method:
        //    - then ensure it is an image
        //    - save it locally with a random 6 character long 62 bit number (a-zA-Z0-9)
        //    - give them the URL

        print!("{:?} ", req);
        let method = req.method().to_string();
        match method.borrow() {
            "GET" => {
                self.handle_get_image_req(req);
            }
            "POST" => {

                {
                    let content_type = req.headers().iter().find(|&header| header.field.equiv("Content-Type"));
                    match content_type {
                        Some(x) => {
                            if x.value != self.content_type {
                                println!("Unsupported content type {}", x.value);
                                return;
                            }

                            // correct format
                            // fall-through
                        },
                        None => {
                            // TODO
                            return;
                        }
                    }
                }

                let data = try_print!(read_whole(req.as_reader()));
                let url = self.upload_image(data.borrow()).unwrap();
                print!("generated url: {}", url);
                let res = Response::from_string(url).with_status_code(StatusCode(200)).with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=UTF-8"[..]).unwrap());
                req.respond(res);
            },
            _ => ()
        }
        println!("");
    }
}
