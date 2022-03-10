use std::fs::{self, File};
use std::io::Read;

use bytes::BytesMut;

use isobmff::IO;

fn main() {
    let s = BytesMut::from((|filename| {
        let mut f = File::open(filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        buffer
    })("avc_fragmented.mp4").as_slice());

    parse(s);
}

fn parse(mut buf: BytesMut) {
    let mut moov = isobmff::moov::moov::default();
    let mut moof = isobmff::moof::moof::default();

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
                moov = isobmff::moov::parse(&mut b.payload);
                eprintln!("{:?}", moov);
            }
            // moof: Movie Fragment
            isobmff::moof::moof::BOX_TYPE => {
                moof = isobmff::moof::parse(&mut b.payload);
                eprintln!("{:?}", moof);
            }
            // mdat: Media Data
            0x6d646174 => {
            }
            _ => {}
        }
    }
}
