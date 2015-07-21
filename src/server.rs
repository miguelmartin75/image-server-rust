use std::io;
use std::io::Write;
use std::fs;
use std::fs::File;
use std::str::FromStr;
use std::borrow::Borrow;

use std::sync::Mutex;

use hyper::{Get, Post};
use hyper::server::{Request, Response};
use hyper::uri::RequestUri::AbsolutePath;
use hyper::header::ContentType;

use hyper::status::StatusCode::{UnsupportedMediaType, InternalServerError, NotFound, BadRequest};

use rand::Rng;
use rand::StdRng;

use image_url::*;
use util::read_whole;

macro_rules! try_print(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { println!("Error: {}", e); return; }
        }
    }}
);

pub struct Server<'a> {
    image_dir : &'a str,
    content_type: ContentType,
    server_url: &'a str,
    run_address: &'a str,
    used_urls : UsedUrlSet,
    mutex : Mutex<UsedUrlSet>,
    rng : StdRng,
}

impl <'a> Server<'a> {
    pub fn new(image_dir: &'a str, content_type: ContentType, run_address: &'a str, server_url: &'a str) -> Server<'a> {
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

        Server { image_dir: image_dir, 
                 content_type: content_type,
                 server_url: server_url,
                 run_address: run_address,
                 used_urls: used_urls, rng: panic!(StdRng::new()),
                 mutex: Mutex::new(used_urls) }
    }

    pub fn run(&self) {
        //hyper::Server::http(self.run_address).unwrap().handle(on_req_recieved).unwrap();
    }

    // retrieves an image path
    // image.png => <image_dir>/image
    // image => <image_dir>/image
    fn get_image_path(&self, path: &str) -> String {
        // remove extension
        let base = path.split(".").nth(0).unwrap();
        self.image_dir.to_string() + base
    }

    fn retrieve_image(&mut self, path: &str) -> io::Result<Vec<u8>> {
        let full_path = self.get_image_path(path);
        match fs::metadata(&full_path) {
            Ok(_) => {
                let mut file = File::open(&full_path).unwrap();
                return read_whole(&mut file);
            }, 
            Err(e) => Err(e)
        }
    }

    fn gen_image_url(&mut self) -> ImageUrl {
        ImageUrl(self.rng.gen::<ImageUrlImpl>())
    }

    fn upload_image(&mut self, data: &[u8]) -> Option<String> {
        let mut num = self.gen_image_url();
        while self.used_urls.contains(&num) {
            num = self.gen_image_url();
        }

        let file_name = self.get_image_path(num.to_string().borrow());

        return match File::create(&file_name) {
            Ok(mut file) => {
                match file.write_all(data) {
                    Ok(_) => Some(self.server_url.to_string() + file_name.borrow()),
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    }

    fn handle_get_image_req(&mut self, path: &str, mut res: Response) {
        match self.retrieve_image(path) {
            Ok(data) => {
                res.headers_mut().set(self.content_type.clone());
                try_print!(res.send(data.borrow()));
            },
            Err(e) => {
                println!("Error: {}", e);
                *res.status_mut() = match e.kind() {
                    io::ErrorKind::NotFound => NotFound,
                    _ => InternalServerError
                }
            }
        }
    }

    fn on_req_recieved(&mut self, mut req: Request, mut res: Response) {
        // determine what request they are making:
        // if it is a GET method, then retrieve the image
        // if it is a POST method:
        //    - then ensure it is an image
        //    - save it locally with a random 6 character long 62 bit number (a-zA-Z0-9)
        //    - give them the URL

        match req.method {
            Get => {
                match req.uri {
                    AbsolutePath(path) => self.handle_get_image_req(&path[..], res),
                    _ => ()
                }
            }
            Post => {
                println!("received post");
                // check to see if the request has a content type
                // and it is something that we can use. If so:
                // then we will upload it 
                if !req.headers.has::<ContentType>() {
                    println!("Error: Post request has no content type");
                    *res.status_mut() = BadRequest;
                    return;
                }

                if *req.headers.get::<ContentType>().unwrap() != self.content_type {
                    println!("UnsupportedMediaType: {}", *req.headers.get::<ContentType>().unwrap());
                    println!("{}", req.headers);
                    *res.status_mut() = UnsupportedMediaType;
                    return;
                }
                
                // give back the URL to the image
                let data = try_print!(read_whole(&mut req));
                let url = self.upload_image(data.borrow());

                // TODO
                //try_print!(res.send(url.borrow().unwrap().as_bytes()));
            },
            _ => ()
        }
    }
}

