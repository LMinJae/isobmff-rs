use std::env;
use std::fs::{self, File};
use std::io::Read;

use bytes::BytesMut;

use isobmff::IO;

fn main() {
    let args: Vec<String> = env::args().collect();
    if 2 > args.len() {
        println!("{:} <FILENAME>", args[0]);
        return
    }

    let filename = &args[1];

    let s = BytesMut::from((|filename| {
        let mut f = File::open(filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        buffer
    })(filename).as_slice());

    parse(s);
}

fn parse(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = isobmff::Object::parse(&mut buf);

        eprintln!("0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // ftyp: File Type
            isobmff::ftyp::ftyp::BOX_TYPE => {
                let ftyp = isobmff::ftyp::parse(&mut b.payload);
                eprintln!("{:?}", ftyp);
            }
            // moov: Movie Box
            isobmff::moov::moov::BOX_TYPE => {
                let moov = isobmff::moov::parse(&mut b.payload);
                eprintln!("{:?}", moov);
            }
            // moof: Movie Fragment
            isobmff::moof::moof::BOX_TYPE => {
                let moof = isobmff::moof::parse(&mut b.payload);
                eprintln!("{:?}", moof);
            }
            // mdat: Media Data
            0x6d646174 => {
            }
            _ => {}
        }
    }
}
