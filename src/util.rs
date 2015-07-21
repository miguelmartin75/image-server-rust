use std::io;
use std::io::Read;

pub fn read_whole<T: Read>(readable: &mut T) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::<u8>::new();

    let result = readable.read_to_end(&mut buffer);
    match result {
        Ok(_) => Ok(buffer),
        Err(e) => Err(e)
    }
}
