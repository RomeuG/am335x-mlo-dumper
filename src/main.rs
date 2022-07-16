use std::io::Seek;
use std::{
    env,
    fs::File,
    io::{BufReader, Read, SeekFrom},
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileOpen(std::io::Error),
    BufRead(std::io::Error),

    Seek(u64),
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong number of arguments. Only the MLO file is required.")
    }

    let file_name = args.get(1).unwrap();
    let file = File::open(file_name).map_err(Error::FileOpen)?;
    let _file_size = file.metadata().unwrap().len();

    let mut buf_reader = BufReader::new(file);

    // seek to presumably CHSETTINGS location
    buf_reader
        .seek(SeekFrom::Start(0x14u64))
        .map_err(|_| Error::Seek(0x14u64))?;

    // check if CHSETTINGS header
    let mut chsettings_array = [0u8; 10];
    buf_reader
        .read_exact(&mut chsettings_array)
        .map_err(Error::BufRead)?;

    let is_chsettings_header = &chsettings_array == b"CHSETTINGS";
    println!("Is CHSETTINGS header: {is_chsettings_header}");

    // rewind
    buf_reader
        .seek(SeekFrom::Start(0u64))
        .map_err(|_| Error::Seek(0u64))?;

    let mut image_size_array = [0u8; 4];
    let mut load_address_array = [0u8; 4];

    if is_chsettings_header {
        // ignore chsettings header
        buf_reader
            .seek(SeekFrom::Start(0x200u64))
            .map_err(|_| Error::Seek(0x200u64))?;

        let image_size = buf_reader
            .read_exact(&mut image_size_array)
            .map(|_| i32::from_le_bytes(image_size_array))
            .map_err(Error::BufRead)? as usize;
        let load_address = buf_reader
            .read_exact(&mut load_address_array)
            .map(|_| i32::from_le_bytes(load_address_array))
            .map_err(Error::BufRead)? as usize;

        println!("Image size: 0x{image_size:x}/{image_size}");
        println!("Load address: 0x{load_address:x}");

        let mut image_vector: Vec<u8> = vec![0u8; image_size];
        buf_reader
            .read_exact(&mut image_vector)
            .map_err(Error::BufRead)?;

        std::fs::write("dump.bin", image_vector).unwrap();
    } else {
        let image_size = buf_reader
            .read_exact(&mut image_size_array)
            .map(|_| i32::from_le_bytes(image_size_array))
            .map_err(Error::BufRead)? as usize;
        let load_address = buf_reader
            .read_exact(&mut load_address_array)
            .map(|_| i32::from_le_bytes(load_address_array))
            .map_err(Error::BufRead)? as usize;

        println!("Image size: 0x{image_size:x}");
        println!("Load address: 0x{load_address:x}");
    }

    Ok(())
}
