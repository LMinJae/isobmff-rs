use std::fs::{self, File};
use std::io::{Read, Write};

use bytes::{Buf, BufMut, BytesMut};

use isobmff::{IO, Object};

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
    let mut f = File::create("./copy.mp4").unwrap();
    let mut moov = isobmff::moov::moov::default();
    let mut moof = isobmff::moof::moof::default();

    let mut offset = 0;

    while 0 < buf.len() {
        let cur_offset = offset;
        let tmp = buf[0];

        let mut b = isobmff::Object::parse(&mut buf);

        eprintln!("[0x{:08x?}] 0x{:08x?}: {:?}", cur_offset, b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        offset += 8 + b.payload.len();
        if 0 == tmp {
            offset += 8;
        }
        match b.box_type {
            // ftyp: File Type
            isobmff::ftyp::ftyp::BOX_TYPE => {
                let mut ftyp = isobmff::ftyp::parse(&mut b.payload);
                eprintln!("{:?}", ftyp);

                f.write_all(Object {
                    box_type: isobmff::ftyp::ftyp::BOX_TYPE,
                    payload: ftyp.as_bytes()
                }.as_bytes().chunk()).unwrap();
            }
            // moov: Movie Box
            isobmff::moov::moov::BOX_TYPE => {
                moov = isobmff::moov::parse(&mut b.payload);
                eprintln!("{:?}", moov);

                f.write_all(Object {
                    box_type: isobmff::moov::moov::BOX_TYPE,
                    payload: moov.as_bytes()
                }.as_bytes().chunk()).unwrap();
            }
            // moof: Movie Fragment
            isobmff::moof::moof::BOX_TYPE => {
                moof_base_offset = cur_offset;
                moof = isobmff::moof::parse(&mut b.payload);
                eprintln!("{:?}", moof);
            }
            // mdat: Media Data
            0x6d646174 => {
                let mut w = BytesMut::with_capacity(16 + moof.len() + b.payload.len());

                w.put(Object {
                    box_type: isobmff::moof::moof::BOX_TYPE,
                    payload: moof.as_bytes()
                }.as_bytes());

                w.put(Object {
                    box_type: 0x6d646174,
                    payload: b.payload
                }.as_bytes());

                f.write_all(w.chunk()).unwrap();
            }
            _ => {}
        }
    }
}
