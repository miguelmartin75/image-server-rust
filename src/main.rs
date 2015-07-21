#![feature(custom_derive)]
extern crate hyper;
extern crate rand;

use std::fmt;

use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashSet;
use std::borrow::Borrow;
use std::str::FromStr;

use rand::Rand;
use rand::Rng;
use rand::StdRng;

use hyper::status::StatusCode::UnsupportedMediaType;
use hyper::status::StatusCode::InternalServerError;
use hyper::status::StatusCode::NotFound;
use hyper::server::{Request, Response};
use hyper::{Get, Post};
use hyper::uri::RequestUri::AbsolutePath;
use hyper::header::ContentType;
use hyper::net::Fresh;
use hyper::server::Handler;

const SERVER_ADDRESS: &'static str = "127.0.0.1:3000";
const SERVER_URL: &'static str = SERVER_ADDRESS; // TODO: change to domain

const IMAGE_DIR: &'static str = "screenies";
const IMAGE_EXT: &'static str = ".png";

fn read_whole<T: Read>(readable: &mut T) -> std::io::Result<Vec<u8>> {
    let mut buffer = Vec::<u8>::new();

    let result = readable.read_to_end(&mut buffer);
    match result {
        Ok(_) => Ok(buffer),
        Err(e) => Err(e)
    }
}

type ImageUrlImpl = u64;

#[derive(PartialEq, Eq, Hash, Debug)]
struct ImageUrl(ImageUrlImpl); 

struct Server<'a> {
    imageDir : &'a str,
    imageExt: &'a str,
    serverUrl: &'a str,
    runAddress: &'a str,
    usedUrls : HashSet<ImageUrl>,
    rng : StdRng,
}

// not sure how to make this into a constant
fn image_content_type() -> ContentType {
    ContentType::png()
}

enum ReadError {
    IoError(std::io::Error),
    NotFound
}

impl <'a> Server<'a> {
    fn new(imageDir: &'a str, imageExt: &'a str, runAddress: &'a str, serverUrl: &'a str) -> Server<'a> {
        let usedUrls = match fs::read_dir(imageDir) {
            Ok(paths) => {
                let mut urls = HashSet::<ImageUrl>::new();
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
            Err(_) => HashSet::<ImageUrl>::new()
        };

        Server { imageDir: imageDir, 
                 imageExt: imageExt, 
                 serverUrl: serverUrl,
                 runAddress: runAddress,
                 usedUrls: usedUrls, rng: panic!(StdRng::new()) }
    }

    fn run<F: Fn(&mut Server, Request, Response) + std::marker::Sync>(&mut self, callback: &F) {
        hyper::Server::http(self.runAddress).unwrap().handle(|req: Request, res: Response| { callback(self, req, res); }).unwrap();
        
    }

    // retrieves an image path
    // image.png => <imageDir>/image.<imageExt>
    // image => <imageDir>/image.<imageExt>
    fn get_image_path(&self, path: &str) -> String {
        // remove extension
        let base = path.split(".").nth(0).unwrap();
        self.imageDir.to_string() + base + self.imageExt
    }

    // returns ReadError if it does not exist or if there's an IO error
    // otherwise returns the data of the image
    fn retrieve_image(&mut self, path: &str) -> Result<Vec<u8>, std::io::Error> {
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
        while self.usedUrls.contains(&num) {
            num = self.gen_image_url();
        }

        let fileName = self.get_image_path(num.to_string().borrow());

        return match File::create(&fileName) {
            Ok(mut file) => {
                file.write_all(data);
                Some(self.serverUrl.to_string() + fileName.borrow())
            }
            Err(e) => None
        }
    }
}


const BASE: u64 = 62;
const IMAGE_URL_CHARS: &'static str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

impl fmt::Display for ImageUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let mut num = self.0;
        let mut result = ['0' as u8; 6];

        for i in (0..result.len()).rev() {
            result[i] = IMAGE_URL_CHARS.as_bytes()[(num % BASE) as usize];
            num /= BASE;
        }

        match std::str::from_utf8(&result) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "")
        }
    }
}

fn index(ch: u8) -> Option<u8> {
    match IMAGE_URL_CHARS.as_bytes().iter().position(|x| *x == ch) {
        Some(x) => Some(x as u8),
        None => None
    }
}

#[derive(Debug)]
struct OutOfRange(u8);

impl FromStr for ImageUrl {
    type Err = OutOfRange;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result: u64 = 0;
        for ch in s.as_bytes().iter() {
            match index(*ch) {
                Some(x) => result = result * BASE + x as u64,
                None => return Err(OutOfRange(*ch))
            }
        }
        return Ok(ImageUrl(result));
    }
}

macro_rules! try_print(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { println!("Error: {}", e); return; }
        }
    }}
);

fn retrieve_image(server: &mut Server, path: &str, mut res: Response) {
    match server.retrieve_image(path) {
        Ok(data) => {
            res.headers_mut().set(image_content_type());
            try_print!(res.send(data.borrow()));
        },
        Err(e) => {
            println!("Error: {}", e);
            *res.status_mut() = match e.kind() {
                std::io::ErrorKind::NotFound => NotFound,
                _ => InternalServerError
            }
        }
    }
}


fn on_req(server: &mut Server, req: Request, res: Response) {
    // determine what request they are making:
    // if it is a GET method, then retrieve the image
    // if it is a POST method:
    //    - then ensure it is an image
    //    - save it locally with a random 6 character long 62 bit number (a-zA-Z0-9)
    //    - give them the URL

    match req.method {
        Get => {
            match req.uri {
                AbsolutePath(path) => retrieve_image(server, &path[..], res),
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
                *res.status_mut() = hyper::BadRequest;
                return;
            }

            if *req.headers.get::<ContentType>().unwrap() != image_content_type() {
                println!("UnsupportedMediaType: {}", *req.headers.get::<ContentType>().unwrap());
                println!("{}", req.headers);
                *res.status_mut() = UnsupportedMediaType;
                return;
            }
            
            // give back the URL to the image
            let data = try_print!(read_whole(&mut req));
            let url = server.upload_image(data.borrow());

            try_print!(res.send(url.borrow().unwrap().as_bytes()));
        },
        _ => ()
    }
}


fn main() {
    let mut server = Server::new(IMAGE_DIR, IMAGE_EXT, SERVER_URL, SERVER_ADDRESS);
    server.run(&on_req);
}
