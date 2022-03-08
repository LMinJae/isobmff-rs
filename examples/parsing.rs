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
            // ftyp: FileTypeBox
            0x66_74_79_70 => {
                let ftyp = fmp4::ftyp::parse(&mut b.payload);
                eprintln!("\tmajor_brand: {:?}", std::str::from_utf8(&ftyp.major_brand.to_be_bytes()).unwrap_or(""));
                eprintln!("\tminor_version: {:?}", ftyp.minor_version);
                eprintln!("\tcompatible_brands: {:?}", ftyp.compatible_brands);
            }
            // moov: MovieBox
            0x6d_6f_6f_76 => {
                parse_moov(b.payload);
            }
            // moof
            0x6d_6f_6f_66 => {
                parse_moof(b.payload);
            }
            // mdat: Media Data
            0x6d_64_61_74 => {
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
            0x6d_76_68_64 => {
                let mvhd = fmp4::mvhd::parse(&mut b.payload);

                eprintln!("\t\tcreation_time: {:?}", mvhd.creation_time);
                eprintln!("\t\tmodification_time: {:?}", mvhd.modification_time);
                eprintln!("\t\ttimescale: {:?}", mvhd.timescale);
                eprintln!("\t\tduration: {:?}", mvhd.duration);

                eprintln!("\t\trate: {:?}", mvhd.rate);
                eprintln!("\t\tvolume: {:?}", mvhd.volume);
                eprintln!("\t\tmatrix: {:?}", mvhd.matrix);
                eprintln!("\t\tnext_track_ID: {:?}", mvhd.next_track_id);
            }
            // trak: Track
            0x74_72_61_6b => {
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
            // tkhd
            0x74_6b_68_64 => {
                let tkhd = fmp4::tkhd::parse(&mut b.payload);

                eprintln!("\t\t\tcreation_time: {:?}", tkhd.creation_time);
                eprintln!("\t\t\tmodification_time: {:?}", tkhd.modification_time);
                eprintln!("\t\t\ttrack_id: {:?}", tkhd.track_id);
                eprintln!("\t\t\t\tduration: {:?}", tkhd.duration);

                eprintln!("\t\t\tlayer: {:?}", tkhd.layer);
                eprintln!("\t\t\talternate_group: {:?}", tkhd.alternate_group);
                eprintln!("\t\t\tvolume: {:?}", tkhd.volume);
                eprintln!("\t\t\twidth: {:?}", tkhd.width);
                eprintln!("\t\t\theight: {:?}", tkhd.height);
            }
            // mdia: Meida
            0x6d_64_69_61 => {
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
            0x6d_64_68_64 => {
                let mdhd = fmp4::mdhd::parse(&mut b.payload);

                eprintln!("\t\t\t\tcreation_time: {:?}", mdhd.creation_time);
                eprintln!("\t\t\t\tmodification_time: {:?}", mdhd.modification_time);
                eprintln!("\t\t\t\ttimescale: {:?}", mdhd.timescale);
                eprintln!("\t\t\t\tduration: {:?}", mdhd.duration);

                eprintln!("\t\t\t\tlanguage: {:?}", mdhd.language);
            }
            // hdlr: Handler Reference
            0x68646c72 => {
                let hdlr = fmp4::hdlr::parse(&mut b.payload);

                eprintln!("\t\t\t\thandler_type: {:?}", hdlr.handler_type);
                eprintln!("\t\t\t\tname: {:?}", hdlr.name);
            }
            // minf: Midia Information
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
                let _ = b.payload.get_u32();

                let graphicmode = b.payload.get_u16();
                let mut opcolor = [0_u16; 3];
                for it in opcolor.iter_mut() {
                    *it = b.payload.get_u16();
                }

                eprintln!("\t\t\t\t\tgraphicmode: {:?}", graphicmode);
                eprintln!("\t\t\t\t\topcolor: {:?}", opcolor);
            }
            // smhd: Sound Media Header
            0x736d6864 => {
                let _ = b.payload.get_u32();

                let balance = b.payload.get_u16();
                let _ = b.payload.get_u16();

                eprintln!("\t\t\t\t\tbalance: {:?}", balance);
            }
            // dinf: Data Information
            0x64696e66 => {
                parse_dinf(b.payload);
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

fn parse_dinf(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t\t\t\t\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // dref: Data Reference
            0x64726566 => {
                let _ = b.payload.get_u32();

                let entry_count = b.payload.get_u32();

                eprintln!("\t\t\t\t\t\t{:?}", entry_count);
                for _ in 0..entry_count {
                    let len = b.payload.get_u32() - 4;
                    parse_dref_entry(b.payload.split_to(len as usize));
                }
            }
            _ => {
            }
        }
    }
}

fn parse_dref_entry(mut b: BytesMut) {
    let _type = b.get_u32();
    eprintln!("\t\t\t\t\t\t0x{:08x?}: {:?}", _type, std::str::from_utf8(&_type.to_be_bytes()).unwrap_or(""));
    match _type {
        // url : URL
        0x75726c20 => {
            let url_ = fmp4::url_::parse(&mut b);

            eprintln!("\t\t\t\t\t\t\tflags: {:?}", url_.base.flags);
        }
        _ => {
        }
    }
}

fn parse_stbl(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t\t\t\t\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // stts: Decoding Time to Sample
            0x73747473 => {
                let _ = b.payload.get_u32();

                let entry_count = b.payload.get_u32();

                eprintln!("\t\t\t\t\t\tentry_count: {:?}", entry_count);
                for _ in 0..entry_count {
                    let sample_count = b.payload.get_u32();
                    let sample_delta = b.payload.get_u32();

                    eprintln!("\t\t\t\t\t\t\tsample_count: {:?}", sample_count);
                    eprintln!("\t\t\t\t\t\t\tsample_delta: {:?}", sample_delta);
                }
            }
            // stsd: Sample Description
            0x73747364 => {
                let _ = b.payload.get_u32();

                let entry_count = b.payload.get_u32();

                eprintln!("\t\t\t\t\t\tentry_count: {:?}", entry_count);
                for _ in 0..entry_count {
                    let len = b.payload.get_u32() - 4;
                    parse_stsd_entry(b.payload.split_to(len as usize));
                }
            }
            // stsc: Sample To Chunk
            0x73747363 => {
                let _ = b.payload.get_u32();

                let entry_count = b.payload.get_u32();

                eprintln!("\t\t\t\t\t\tentry_count: {:?}", entry_count);
                for _ in 0..entry_count {
                    let first_chunk = b.payload.get_u32();
                    let samples_per_chunk = b.payload.get_u32();
                    let sample_description_index = b.payload.get_u32();

                    eprintln!("\t\t\t\t\t\t\tfirst_chunk: {:?}", first_chunk);
                    eprintln!("\t\t\t\t\t\t\tsamples_per_chunk: {:?}", samples_per_chunk);
                    eprintln!("\t\t\t\t\t\t\tsample_description_index: {:?}", sample_description_index);
                }
            }
            // stsz: Sample Size
            0x7374737a => {
                let _ = b.payload.get_u32();

                let sample_size = b.payload.get_u32();
                let sample_count = b.payload.get_u32();

                eprintln!("\t\t\t\t\t\tsample_size: {:?}", sample_size);
                eprintln!("\t\t\t\t\t\tsample_count: {:?}", sample_count);
                if 0 == sample_size {
                    for _ in 0..sample_count {
                        let entry_size = b.payload.get_u32();

                        eprintln!("\t\t\t\t\t\t\tfirst_chunk: {:?}", entry_size);
                    }
                }
            }
            // stco: Chunk Offset
            0x7374636f => {
                let _ = b.payload.get_u32();

                let entry_count = b.payload.get_u32();

                eprintln!("\t\t\t\t\t\tentry_count: {:?}", entry_count);
                for _ in 0..entry_count {
                    let chunk_offset = b.payload.get_u32();

                    eprintln!("\t\t\t\t\t\t\tchunk_offset: {:?}", chunk_offset);
                }
            }
            _ => {
            }
        }
    }
}

fn parse_stsd_entry(mut b: BytesMut) {
    let format = b.get_u32();
    eprintln!("\t\t\t\t\t\t\tformat: 0x{:08x?}: {:?}", format, std::str::from_utf8(&format.to_be_bytes()).unwrap_or(""));
    let _ = b.split_to(6);
    let data_reference_index = b.get_u16();
    eprintln!("\t\t\t\t\t\t\t\tdata_reference_index: {:?}", data_reference_index);

    match format {
        // avc1
        0x61766331 => {
            let _ = b.get_u16();
            let _ = b.get_u16();
            let _ = b.split_to(12);
            let width = b.get_u16();
            let height = b.get_u16();
            let horizresolution = b.get_u32();
            let vertresolution = b.get_u32();
            let _ = b.get_u32();
            let frame_count = b.get_u16();
            let compressorname = b.split_to(32);
            let depth = b.get_u16();
            let _ = b.get_u16();

            eprintln!("\t\t\t\t\t\t\t\twidth: {:?}", width);
            eprintln!("\t\t\t\t\t\t\t\theight: {:?}", height);
            eprintln!("\t\t\t\t\t\t\t\thorizresolution: {:?}", horizresolution);
            eprintln!("\t\t\t\t\t\t\t\tvertresolution: {:?}", vertresolution);
            eprintln!("\t\t\t\t\t\t\t\tframe_count: {:?}", frame_count);
            eprintln!("\t\t\t\t\t\t\t\tcompressorname: {:?}", compressorname);
            eprintln!("\t\t\t\t\t\t\t\tdepth: {:?}", depth);

            parse_avc1(b);
        }
        _ => {
        }
    }
}

fn parse_avc1(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t\t\t\t\t\t\t\t\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // avcC
            0x61766343 => {
                let configuration_version = b.payload.get_u8();
                let profile_indication = b.payload.get_u8();
                let profile_compatibility = b.payload.get_u8();
                let level_indication = b.payload.get_u8();
                let length_size_minus_one = b.payload.get_u8() & 0b11;
                eprintln!("\t\t\t\t\t\t\t\t\t\tconfiguration_version: {:?}", configuration_version);
                eprintln!("\t\t\t\t\t\t\t\t\t\tprofile_indication: {:?}", profile_indication);
                eprintln!("\t\t\t\t\t\t\t\t\t\tprofile_compatibility: {:?}", profile_compatibility);
                eprintln!("\t\t\t\t\t\t\t\t\t\tlevel_indication: {:?}", level_indication);
                eprintln!("\t\t\t\t\t\t\t\t\t\tlength_size_minus_one: {:?}", length_size_minus_one);
                let nb_sps = b.payload.get_u8() & 0b11111;
                eprintln!("\t\t\t\t\t\t\t\t\t\tnb_sps: {:?}", nb_sps);
                for _ in 0..nb_sps {
                    let len = b.payload.get_u16();
                    eprintln!("\t\t\t\t\t\t\t\t\t\t\t{:x?}", b.payload.split_to(len as usize));
                }
                let nb_pps = b.payload.get_u8() & 0b11111;
                eprintln!("\t\t\t\t\t\t\t\t\t\tnb_pps: {:?}", nb_pps);
                for _ in 0..nb_pps {
                    let len = b.payload.get_u16();
                    eprintln!("\t\t\t\t\t\t\t\t\t\t\t{:x?}", b.payload.split_to(len as usize));
                }
            }
            _ => {
            }
        }
    }
}

fn parse_moof(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // mfhd:
            0x6d666864 => {
                let _ = b.payload.get_u32();

                let sequence_number = b.payload.get_u32();
                eprintln!("\t\tsequence_number: {:?}", sequence_number);
            }
            // traf:
            0x74726166 => {
                parse_traf(b.payload);
            }
            _ => {
                eprintln!("\t\t{:?}", b.payload);
            }
        }
    }
}

fn parse_traf(mut buf: BytesMut) {
    while 0 < buf.len() {
        let mut b = fmp4::Box::parse(&mut buf);

        eprintln!("\t\t0x{:08x?}: {:?}", b.box_type, std::str::from_utf8(&b.box_type.to_be_bytes()).unwrap_or(""));
        match b.box_type {
            // tfhd:
            0x74666864 => {
                let flags = b.payload.get_u32();
                eprintln!("\t\t\tflags: {:?}", flags);

                let track_id = b.payload.get_u32();

                eprintln!("\t\t\ttrack_id: {:?}", track_id);

                // optional
                if 0 != (0x000001 & flags) {
                    let base_data_offset = b.payload.get_u64();
                    eprintln!("\t\t\tbase_data_offset: {:?}", base_data_offset);
                }
                if 0 != (0x000002 & flags) {
                    let sample_description_index = b.payload.get_u32();
                    eprintln!("\t\t\tsample_description_index: {:?}", sample_description_index);
                }
                if 0 != (0x010000 & flags) || 0 != (0x000008 & flags) {
                    let default_sample_duration = b.payload.get_u32();
                    eprintln!("\t\t\tdefault_sample_duration: {:?}", default_sample_duration);
                }
                if 0 != (0x000010 & flags) {
                    let default_sample_size = b.payload.get_u32();
                    eprintln!("\t\t\tdefault_sample_size: {:?}", default_sample_size);
                }
                if 0 != (0x000020 & flags) {
                    let default_sample_flags = b.payload.get_u32();
                    eprintln!("\t\t\tdefault_sample_flags: {:?}", default_sample_flags);
                }
            }
            // trun: Track Fragment Run
            0x7472756e => {
                let flags = b.payload.get_u32() & 0x00ffffff;
                eprintln!("\t\t\tflags: {:?}", flags);

                let sample_count = b.payload.get_u32();
                eprintln!("\t\t\tsample_count: {:?}", sample_count);
                if 0 != (0x000001 & flags) {
                    let data_offset = b.payload.get_i32();
                    eprintln!("\t\t\tdata_offset: {:?}", data_offset);
                }
                if 0 != (0x000004 & flags) {
                    let first_sample_flags = b.payload.get_u32();
                    eprintln!("\t\t\tfirst_sample_flags: {:?}", first_sample_flags);
                }

                for _ in 0..sample_count {
                    eprintln!();
                    if 0 != (0x000100 & flags) {
                        let sample_duration = b.payload.get_u32();
                        eprintln!("\t\t\tsample_duration: {:?}", sample_duration);
                    }
                    if 0 != (0x000200 & flags) {
                        let sample_size = b.payload.get_u32();
                        eprintln!("\t\t\tsample_size: {:?}", sample_size);
                    }
                    if 0 != (0x000400 & flags) {
                        let sample_flags = b.payload.get_u32();
                        eprintln!("\t\t\tsample_flags: {:?}", sample_flags);
                    }
                    if 0 != (0x000800 & flags) {
                        let sample_composition_time_offset = b.payload.get_u32();
                        eprintln!("\t\t\tsample_composition_time_offset: {:?}", sample_composition_time_offset);
                    }
                }
            }
            // tfdt:
            0x74666474 => {
                let version = b.payload.get_u32() >> 24;

                if 1 == version {
                    let base_media_decode_time = b.payload.get_u64();
                    eprintln!("\t\t\tbase_media_decode_time: {:?}", base_media_decode_time);
                }
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
