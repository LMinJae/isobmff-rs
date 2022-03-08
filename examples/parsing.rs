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
                parse_mdia(b.payload);
            }
            _ => {
            }
        }
    }
}

fn parse_mdia(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t\t\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // mdhd: Media Header
            0x6d646864 => {
                let mdhd = fmp4::mdhd::parse(&mut b.payload);

                eprintln!("{:?}", mdhd);
            }
            // hdlr: Handler Reference
            0x68646c72 => {
                let hdlr = fmp4::hdlr::parse(&mut b.payload);

                eprintln!("{:?}", hdlr);
            }
            // minf: Media Information
            0x6d696e66 => {
                parse_minf(b.payload);
            }
            _ => {
            }
        }
    }
}

fn parse_minf(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t\t\t\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // vmhd: Video Media Header
            0x766d6864 => {
                let vmhd = fmp4::vmhd::parse(&mut b.payload);

                eprintln!("{:?}", vmhd);
            }
            // smhd: Sound Media Header
            0x736d6864 => {
                let smhd = fmp4::smhd::parse(&mut b.payload);
                eprintln!("{:?}", smhd);
            }
            // dinf: Data Information
            0x64696e66 => {
                let dinf = fmp4::dinf::parse(&mut b.payload);
                eprintln!("{:?}", dinf);
            }
            // stbl: Sample Table
            0x7374626c => {
                parse_stbl(b.payload);
            }
            _ => {
            }
        }
    }
}

fn parse_stbl(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t\t\t\t\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // stsd: Sample Description
            0x73747364 => {
                let stsd = fmp4::stsd::parse(&mut b.payload);
                eprintln!("{:?}", stsd);
            }
            // stts: Decoding Time to Sample
            0x73747473 => {
                let stts = fmp4::stts::parse(&mut b.payload);
                eprintln!("{:?}", stts);
            }
            // stsc: Sample To Chunk
            0x73747363 => {
                let stsc = fmp4::stsc::parse(&mut b.payload);
                eprintln!("{:?}", stsc);
            }
            // stsz: Sample Size
            0x7374737a => {
                let stsz = fmp4::stsz::parse(&mut b.payload);
                eprintln!("{:?}", stsz);
            }
            // stco: Chunk Offset
            0x7374636f => {
                let stco = fmp4::stco::parse(&mut b.payload);
                eprintln!("{:?}", stco);
            }
            _ => {
            }
        }
    }
}

fn parse_mdat(mut buf: BytesMut) {
    eprintln!("AVC");
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

    eprintln!("AAC");
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
    eprintln!("{:02x?}", buf.chunk());
}

fn parse_aac(buf: BytesMut) {
    eprintln!("{:02x?}", buf.chunk());
}
