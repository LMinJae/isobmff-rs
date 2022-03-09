use std::fs::{self, File};
use std::io::{Read};
use bytes::{Buf, BytesMut};
use fmp4::IO;

fn main() {
    let s = BytesMut::from((|filename| {
        let mut f = File::open(filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        buffer
    })("bipbop-fragmented.mp4").as_slice());

    parse(s);
}

fn parse(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // ftyp: File Type
            0x66747970 => {
                let ftyp = fmp4::ftyp::parse(&mut b.payload);
                eprintln!("{:?}", ftyp);
            }
            // moov: Movie Box
            0x6d6f6f76 => {
                parse_moov(b.payload);
            }
            // moof: Movie Fragment
            0x6d6f6f66 => {
                let moof = fmp4::moof::parse(&mut b.payload);
                eprintln!("{:?}", moof);
            }
            // mdat: Media Data
            0x6d646174 => {
                parse_mdat(b.payload);
                return
            }
            _ => {
            }
        }
    }
}

fn parse_moov(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // mvhd: Movie Header
            0x6d766864 => {
                let mvhd = fmp4::mvhd::parse(&mut b.payload);
                eprintln!("{:?}", mvhd);
            }
            // trak: Track
            0x7472616b => {
                parse_trak(b.payload);
            }
            _ => {
            }
        }
    }
}

fn parse_trak(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // tkhd: Track Header
            0x746b6864 => {
                let tkhd = fmp4::tkhd::parse(&mut b.payload);
                eprintln!("{:?}", tkhd);
            }
            // mdia: Meida
            0x6d646961 => {
                let mdia = fmp4::mdia::parse(&mut b.payload);
                eprintln!("{:?}", mdia);
            }
            _ => {
            }
        }
    }
}

fn parse_mdat(mut buf: BytesMut) {
    eprintln!("\tAVC");
    parse_avc(buf.split_to(9814));
    parse_avc(buf.split_to(817));
    parse_avc(buf.split_to(598));
    parse_avc(buf.split_to(656));
    parse_avc(buf.split_to(506));
    parse_avc(buf.split_to(703));
    parse_avc(buf.split_to(437));
    parse_avc(buf.split_to(550));
    parse_avc(buf.split_to(459));
    parse_avc(buf.split_to(1008));
    parse_avc(buf.split_to(431));
    parse_avc(buf.split_to(723));
    parse_avc(buf.split_to(475));
    parse_avc(buf.split_to(607));
    parse_avc(buf.split_to(509));
    parse_avc(buf.split_to(680));
    parse_avc(buf.split_to(428));
    parse_avc(buf.split_to(584));
    parse_avc(buf.split_to(473));
    parse_avc(buf.split_to(891));
    parse_avc(buf.split_to(421));
    parse_avc(buf.split_to(636));
    parse_avc(buf.split_to(440));
    parse_avc(buf.split_to(562));

    eprintln!("\tAAC");
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(169));
    parse_aac(buf.split_to(145));
    parse_aac(buf.split_to(24));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
    parse_aac(buf.split_to(6));
}

fn parse_avc(buf: BytesMut) {
    eprintln!("\t\t{:02x?}", buf.chunk());
}

fn parse_aac(buf: BytesMut) {
    eprintln!("\t\t{:02x?}", buf.chunk());
}
