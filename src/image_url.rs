use std::fmt;
use std::str;
use std::str::FromStr;

use std::collections::HashSet;

pub type ImageUrlImpl = u64;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ImageUrl(pub ImageUrlImpl); 

#[derive(Debug)]
pub struct OutOfRange(u8);

pub type UsedUrlSet = HashSet<ImageUrl>;

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

        match str::from_utf8(&result) {
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
