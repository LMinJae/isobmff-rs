mod avc;

use std::cmp::min;
use std::fmt::{Debug, Formatter};
use bytes::{Buf, BufMut, BytesMut};
use crate::{FullBox, IO, Object};
use crate::moov::avc::avcC;

pub fn parse(r: &mut BytesMut) -> moov {
    moov::parse(r)
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct moov {
    mvhd: mvhd,
    traks: Vec<trak>,
}

impl moov {
    pub const BOX_TYPE: u32 = 0x6d6f6f76;
}

impl Default for moov {
    fn default() -> Self {
        Self {
            mvhd: Default::default(),
            traks: vec![]
        }
    }
}

impl Debug for moov {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t0x{:08x?}: \"mvhd\"\n", mvhd::BOX_TYPE))?;
        self.mvhd.fmt(f)?;
        for it in &self.traks {
            f.write_fmt(format_args!("\n\t0x{:08x?}: \"trak\"\n", trak::BOX_TYPE))?;
            it.fmt(f)?;
        }

        Ok(())
    }
}

impl IO for moov {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();

        while 0 < r.len() {
            let mut b = Object::parse(r);
            match b.box_type {
                // mvhd: Movie Header
                0x6d766864 => {
                    rst.mvhd = mvhd::parse(&mut b.payload);
                }
                // trak: Track
                0x7472616b => {
                    rst.traks.push(trak::parse(&mut b.payload));
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: 0x6d766864,
            payload: self.mvhd.as_bytes(),
        }.as_bytes());
        for it in self.traks.iter_mut() {
            w.put(Object {
                box_type: 0x7472616b,
                payload: it.as_bytes(),
            }.as_bytes());
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct mvhd {
    creation_time: u64,
    modification_time: u64,
    timescale: u32,
    duration: u64,
    rate: u32,
    volume: u16,
    matrix: [u32; 9],
    next_track_id: u32,
}

impl mvhd {
    pub const BOX_TYPE: u32 = 0x6d766864;
}

impl Default for mvhd {
    //! extends FullBox(‘mvhd’, version, 0) {
    //!     if (version==1) {
    //!         unsigned int(64) creation_time;
    //!         unsigned int(64) modification_time;
    //!         unsigned int(32) timescale;
    //!         unsigned int(64) duration;
    //!     } else { // version==0
    //!         unsigned int(32) creation_time;
    //!         unsigned int(32) modification_time;
    //!         unsigned int(32) timescale;
    //!         unsigned int(32) duration;
    //!     }
    //!     template int(32) rate = 0x00010000; // typically 1.0
    //!     template int(16) volume = 0x0100; // typically, full volume
    //!     const bit(16) reserved = 0;
    //!     const unsigned int(32)[2] reserved = 0;
    //!     template int(32)[9] matrix = { 0x00010000,0,0,0,0x00010000,0,0,0,0x40000000 };
    //!     // Unity matrix
    //!     bit(32)[6] pre_defined = 0;
    //!     unsigned int(32) next_track_ID;
    //! }
    fn default() -> Self {
        Self {
            creation_time: 0,
            modification_time: 0,
            timescale: 0,
            duration: 0,
            rate: 0x00010000,
            volume: 0x0100,
            matrix: [0x00010000,0,0,0,0x00010000,0,0,0,0x40000000],
            next_track_id: 0,
        }
    }
}

impl Debug for mvhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\tcreation_time: {:?}", self.creation_time))?;
        f.write_fmt(format_args!("\n\t\tmodification_time: {:?}", self.modification_time))?;
        f.write_fmt(format_args!("\n\t\ttimescale: {:?}", self.timescale))?;
        f.write_fmt(format_args!("\n\t\tduration: {:?}", self.duration))?;
        f.write_fmt(format_args!("\n\t\trate: 0x{:08x?}", self.rate))?;
        f.write_fmt(format_args!("\n\t\tvolume: 0x{:04x?}", self.volume))?;
        f.write_fmt(format_args!("\n\t\tmatrix: ["))?;
        for i in 0..9 {
            if 0 == i % 3 {
                f.write_fmt(format_args!("\n\t\t\t"))?;
            }
            f.write_fmt(format_args!("0x{:08x?}, ",self.matrix[i]))?;
        }
        f.write_fmt(format_args!("\n\t\t]"))?;
        f.write_fmt(format_args!("\n\t\tnext_track_ID: {:?}", self.next_track_id))?;

        Ok(())
    }
}

impl IO for mvhd {
    fn parse(r: &mut BytesMut) -> Self {
        let version = r.get_u8();
        let _flags = r.split_to(3);

        let mut rst = Self::default();

        {
            let (
                creation_time,
                modification_time,
                timescale,
                duration,
            ) = if 1 == version {
                (
                    r.get_u64(),
                    r.get_u64(),
                    r.get_u32(),
                    r.get_u64(),
                )
            } else {
                (
                    r.get_u32() as u64,
                    r.get_u32() as u64,
                    r.get_u32(),
                    r.get_u32() as u64,
                )
            };
            rst.creation_time = creation_time;
            rst.modification_time = modification_time;
            rst.timescale = timescale;
            rst.duration = duration;
        }

        rst.rate = r.get_u32();
        rst.volume = r.get_u16();
        let _ = r.get_u16();
        let _ = r.get_u64();
        for it in rst.matrix.iter_mut() {
            *it = r.get_u32();
        }
        let _ = r.split_to(24);
        rst.next_track_id = r.get_u32();

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        if (u32::MAX as u64) < self.creation_time ||
            (u32::MAX as u64) < self.modification_time ||
            (u32::MAX as u64) < self.duration {
            w.put_u32(0x01000000);

            w.put_u64(self.creation_time);
            w.put_u64(self.modification_time);
            w.put_u32(self.timescale);
            w.put_u64(self.duration);
        } else {
            w.put_u32(0);

            w.put_u32(self.creation_time as u32);
            w.put_u32(self.modification_time as u32);
            w.put_u32(self.timescale);
            w.put_u32(self.duration as u32);
        }

        w.put_u32(self.rate);
        w.put_u16(self.volume);
        w.put_u16(0);
        w.put_u64(0);
        for it in self.matrix {
            w.put_u32(it);
        }
        w.put_u64(0);
        w.put_u64(0);
        w.put_u64(0);
        w.put_u32(self.next_track_id);

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct trak {
    tkhd: tkhd,
    mdia: mdia,
}

impl trak {
    pub const BOX_TYPE: u32 = 0x7472616b;
}

impl Default for trak {
    fn default() -> Self {
        Self {
            tkhd: Default::default(),
            mdia: Default::default(),
        }
    }
}

impl Debug for trak {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t0x{:08x?}: \"tkhd\"\n", tkhd::BOX_TYPE))?;
        self.tkhd.fmt(f)?;
        f.write_fmt(format_args!("\n\t\t0x{:08x?}: \"mdia\"\n", mdia::BOX_TYPE))?;
        self.mdia.fmt(f)?;

        Ok(())
    }
}

impl IO for trak {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();

        while 0 < r.len() {
            let mut b = Object::parse(r);

            match b.box_type {
                // tkhd: Track Header
                0x746b6864 => {
                    rst.tkhd = tkhd::parse(&mut b.payload);
                }
                // mdia: Meida
                0x6d646961 => {
                    rst.mdia = mdia::parse(&mut b.payload);
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: 0x746b6864,
            payload: self.tkhd.as_bytes(),
        }.as_bytes());
        w.put(Object {
            box_type: 0x6d646961,
            payload: self.mdia.as_bytes(),
        }.as_bytes());

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct tkhd {
    base: FullBox,

    creation_time: u64,
    modification_time: u64,
    track_id: u32,
    duration: u64,
    layer: u16,
    alternate_group: u16,
    volume: u16,
    matrix: [u32; 9],
    width: u32,
    height: u32,
}

impl tkhd {
    pub const BOX_TYPE: u32 = 0x746b6864;
}

mod tkhd_flags {
    pub(crate) const TRACK_ENABLED: u32 = 0x000001;
    pub(crate) const TRACK_IN_MOVIE: u32 = 0x000002;
    pub(crate) const TRACK_IN_PREVIEW: u32 = 0x000004;
    const TRACK_SIZE_IS_ASPECT_RATIO: u32 = 0x000008;
}

impl Default for tkhd {
    //! extends FullBox(‘tkhd’, version, flags){
    //!     if (version==1) {
    //!         unsigned int(64) creation_time;
    //!         unsigned int(64) modification_time;
    //!         unsigned int(32) track_ID;
    //!         const unsigned int(32) reserved = 0;
    //!         unsigned int(64) duration;
    //!     } else { // version==0
    //!         unsigned int(32) creation_time;
    //!         unsigned int(32) modification_time;
    //!         unsigned int(32) track_ID;
    //!         const unsigned int(32) reserved = 0;
    //!         unsigned int(32) duration;
    //!     }
    //!     const unsigned int(32)[2] reserved = 0;
    //!     template int(16) layer = 0;
    //!     template int(16) alternate_group = 0;
    //!     template int(16) volume = {if track_is_audio 0x0100 else 0};
    //!     const unsigned int(16) reserved = 0;
    //!     template int(32)[9] matrix= { 0x00010000,0,0,0,0x00010000,0,0,0,0x40000000 };
    //!     // unity matrix
    //!     unsigned int(32) width;
    //!     unsigned int(32) height;
    //! }
    fn default() -> Self {
        Self {
            base: FullBox { version: 0, flags: tkhd_flags::TRACK_ENABLED | tkhd_flags::TRACK_IN_MOVIE | tkhd_flags::TRACK_IN_PREVIEW },
            creation_time: 0,
            modification_time: 0,
            track_id: 0,
            duration: 0,
            layer: 0,
            alternate_group: 0,
            volume: 0,
            matrix: [0x00010000,0,0,0,0x00010000,0,0,0,0x40000000],
            width: 0,
            height: 0,
        }
    }
}

impl Debug for tkhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\tcreation_time: {:?}", self.creation_time))?;
        f.write_fmt(format_args!("\n\t\t\tmodification_time: {:?}", self.modification_time))?;
        f.write_fmt(format_args!("\n\t\t\ttrack_id: {:?}", self.track_id))?;
        f.write_fmt(format_args!("\n\t\t\tduration: {:?}", self.duration))?;
        f.write_fmt(format_args!("\n\t\t\tlayer: {:?}", self.layer))?;
        f.write_fmt(format_args!("\n\t\t\talternate_group: {:?}", self.alternate_group))?;
        f.write_fmt(format_args!("\n\t\t\tvolume: {:?}", self.volume))?;
        f.write_fmt(format_args!("\n\t\tmatrix: ["))?;
        for i in 0..9 {
            if 0 == i % 3 {
                f.write_fmt(format_args!("\n\t\t\t"))?;
            }
            f.write_fmt(format_args!("0x{:08x?}, ",self.matrix[i]))?;
        }
        f.write_fmt(format_args!("\n\t\t]"))?;
        f.write_fmt(format_args!("\n\t\t\twidth: {:?}", self.width))?;
        f.write_fmt(format_args!("\n\t\t\theight: {:?}", self.height))?;

        Ok(())
    }
}

impl IO for tkhd {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();
        rst.base = FullBox::parse(r);

        {
            let (
                creation_time,
                modification_time,
                track_id,
                _,
                duration,
            ) = if 1 == rst.base.version {
                (
                    r.get_u64(),
                    r.get_u64(),
                    r.get_u32(),
                    r.get_u32(),
                    r.get_u64(),
                )
            } else {
                (
                    r.get_u32() as u64,
                    r.get_u32() as u64,
                    r.get_u32(),
                    r.get_u32(),
                    r.get_u32() as u64,
                )
            };
            rst.creation_time = creation_time;
            rst.modification_time = modification_time;
            rst.track_id = track_id;
            rst.duration = duration;
        }

        let _ = r.get_u64();
        rst.layer = r.get_u16();
        rst.alternate_group = r.get_u16();
        rst.volume = r.get_u16();
        let _ = r.get_u16();
        for it in rst.matrix.iter_mut() {
            *it = r.get_u32();
        }
        rst.width = r.get_u32();
        rst.height = r.get_u32();

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        if (u32::MAX as u64) < self.creation_time ||
            (u32::MAX as u64) < self.modification_time ||
            (u32::MAX as u64) < self.duration {
            self.base.version = 1;
            w.put(self.base.as_bytes());

            w.put_u64(self.creation_time);
            w.put_u64(self.modification_time);
            w.put_u32(self.track_id);
            w.put_u32(0);
            w.put_u64(self.duration);
        } else {
            self.base.version = 0;
            w.put(self.base.as_bytes());

            w.put_u32(self.creation_time as u32);
            w.put_u32(self.modification_time as u32);
            w.put_u32(self.track_id);
            w.put_u32(0);
            w.put_u32(self.duration as u32);
        }

        w.put_u64(0);
        w.put_u16(self.layer);
        w.put_u16(self.alternate_group);
        w.put_u16(self.volume);
        w.put_u16(0);
        for it in self.matrix {
            w.put_u32(it);
        }
        w.put_u32(self.width);
        w.put_u32(self.height);

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct mdia {
    mdhd: mdhd,
    hdlr: hdlr,
    minf: minf,
}

impl mdia {
    pub const BOX_TYPE: u32 = 0x6d646961;
}

impl Default for mdia {
    fn default() -> Self {
        Self {
            mdhd: Default::default(),
            hdlr: Default::default(),
            minf: Default::default(),
        }
    }
}

impl Debug for mdia {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t0x{:08x?}: \"mdhd\"\n", mdhd::BOX_TYPE))?;
        self.mdhd.fmt(f)?;
        f.write_fmt(format_args!("\n\t\t\t0x{:08x?}: \"hdlr\"\n", hdlr::BOX_TYPE))?;
        self.hdlr.fmt(f)?;
        f.write_fmt(format_args!("\n\t\t\t0x{:08x?}: \"minf\"\n", minf::BOX_TYPE))?;
        self.minf.fmt(f)?;

        Ok(())
    }
}

impl IO for mdia {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();

        while 0 < r.len() {
            let mut b = Object::parse(r);

            match b.box_type {
                // mdhd: Media Header
                0x6d646864 => {
                    rst.mdhd = mdhd::parse(&mut b.payload);
                }
                // hdlr: Handler Reference
                0x68646c72 => {
                    rst.hdlr = hdlr::parse(&mut b.payload);
                }
                // minf: Media Information
                0x6d696e66 => {
                    rst.minf = minf::parse(&mut b.payload);
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: 0x6d646864,
            payload: self.mdhd.as_bytes(),
        }.as_bytes());
        w.put(Object {
            box_type: 0x68646c72,
            payload: self.hdlr.as_bytes(),
        }.as_bytes());
        w.put(Object {
            box_type: 0x6d696e66,
            payload: self.minf.as_bytes(),
        }.as_bytes());

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct mdhd {
    base: FullBox,

    creation_time: u64,
    modification_time: u64,
    timescale: u32,
    duration: u64,
    language: u16,
}

impl mdhd {
    pub const BOX_TYPE: u32 = 0x6d646864;
}

impl Default for mdhd {
    //! extends FullBox(‘mdhd’, version, 0) {
    //!     if (version==1) {
    //!         unsigned int(64) creation_time;
    //!         unsigned int(64) modification_time;
    //!         unsigned int(32) timescale;
    //!         unsigned int(64) duration;
    //!     } else { // version==0
    //!         unsigned int(32) creation_time;
    //!         unsigned int(32) modification_time;
    //!         unsigned int(32) timescale;
    //!         unsigned int(32) duration;
    //!     }
    //!     bit(1) pad = 0;
    //!     unsigned int(5)[3] language; // ISO-639-2/T language code
    //!     unsigned int(16) pre_defined = 0;
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            creation_time: 0,
            modification_time: 0,
            timescale: 0,
            duration: 0,
            language: 0,
        }
    }
}

impl Debug for mdhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\tcreation_time: {:?}", self.creation_time))?;
        f.write_fmt(format_args!("\n\t\t\t\tmodification_time: {:?}", self.modification_time))?;
        f.write_fmt(format_args!("\n\t\t\t\ttimescale: {:?}", self.timescale))?;
        f.write_fmt(format_args!("\n\t\t\t\tduration: {:?}", self.duration))?;
        f.write_fmt(format_args!("\n\t\t\t\tlanguage: "))?;
        f.write_fmt(format_args!("{:}", String::from_utf8_lossy(&[
            0x60 + (0b11111 & (self.language >> 10)) as u8,
            0x60 + (0b11111 & (self.language >> 5)) as u8,
            0x60 + (0b11111 & self.language) as u8,
        ])))?;

        Ok(())
    }
}

impl IO for mdhd {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            creation_time: 0,
            modification_time: 0,
            timescale: 0,
            duration: 0,
            language: 0
        };

        {
            let (
                creation_time,
                modification_time,
                timescale,
                duration,
            ) = if 1 == rst.base.version {
                (
                    r.get_u64(),
                    r.get_u64(),
                    r.get_u32(),
                    r.get_u64(),
                )
            } else {
                (
                    r.get_u32() as u64,
                    r.get_u32() as u64,
                    r.get_u32(),
                    r.get_u32() as u64,
                )
            };
            rst.creation_time = creation_time;
            rst.modification_time = modification_time;
            rst.timescale = timescale;
            rst.duration = duration;
        }

        rst.language = r.get_u16();
        let _ = r.get_u16();

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        if (u32::MAX as u64) < self.creation_time ||
            (u32::MAX as u64) < self.modification_time ||
            (u32::MAX as u64) < self.duration {
            self.base.version = 1;
            w.put(self.base.as_bytes());

            w.put_u64(self.creation_time);
            w.put_u64(self.modification_time);
            w.put_u32(self.timescale);
            w.put_u64(self.duration);
        } else {
            self.base.version = 0;
            w.put(self.base.as_bytes());

            w.put_u32(self.creation_time as u32);
            w.put_u32(self.modification_time as u32);
            w.put_u32(self.timescale);
            w.put_u32(self.duration as u32);
        }

        w.put_u16(self.language);
        w.put_u16(0);

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct hdlr {
    base: FullBox,

    handler_type: u32,
    name: String,
}

impl hdlr {
    pub const BOX_TYPE: u32 = 0x68646c72;
}

impl Default for hdlr {
    //! extends FullBox(‘hdlr’, version = 0, 0) {
    //!     unsigned int(32) pre_defined = 0;
    //!     unsigned int(32) handler_type;
    //!     const unsigned int(32)[3] reserved = 0;
    //!     string name;
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            handler_type: 0,
            name: "".to_owned(),
        }
    }
}

impl Debug for hdlr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Ok(str) = std::str::from_utf8(&self.handler_type.to_be_bytes()) {
            f.write_fmt(format_args!("\t\t\t\thandler_type: {:?}", str))?;
        } else {
            f.write_fmt(format_args!("\t\t\t\thandler_type: 0x{:08x?}", self.handler_type))?;
        }
        f.write_fmt(format_args!("\n\t\t\t\tname: {:?}", self.name))?;

        Ok(())
    }
}

impl IO for hdlr {
    fn parse(r: &mut BytesMut) -> Self {
        let base = FullBox::parse(r);

        let _ = r.get_u32();
        let handler_type = r.get_u32();
        let _ = r.split_to(12);
        let name = std::str::from_utf8(r.split_to(r.len()).chunk()).unwrap().to_owned();

        Self {
            base,
            handler_type,
            name,
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(0);
        w.put_u32(self.handler_type);
        w.put_u32(0);
        w.put_u32(0);
        w.put_u32(0);
        w.put(self.name.as_bytes());

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct minf {
    mhd: MediaInformationHeader,
    dinf: dinf,
    stbl: stbl,
}

impl minf {
    pub const BOX_TYPE: u32 = 0x6d696e66;
}

impl Default for minf {
    fn default() -> Self {
        Self {
            mhd: MediaInformationHeader::Unknown,
            dinf: Default::default(),
            stbl: Default::default(),
        }
    }
}

impl Debug for minf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.mhd {
            MediaInformationHeader::Unknown => {}
            MediaInformationHeader::vmhd(v) => {
                f.write_fmt(format_args!("\t\t\t\t0x{:08x?}: \"vmhd\"\n", vmhd::BOX_TYPE))?;
                v.fmt(f)?;
            }
            MediaInformationHeader::smhd(v) => {
                f.write_fmt(format_args!("\t\t\t\t0x{:08x?}: \"smhd\"\n", smhd::BOX_TYPE))?;
                v.fmt(f)?;
            }
            MediaInformationHeader::hmhd(v) => {
                f.write_fmt(format_args!("\t\t\t\t0x{:08x?}: \"hmhd\"\n", hmhd::BOX_TYPE))?;
                v.fmt(f)?;
            }
            MediaInformationHeader::nmhd(v) => {
                f.write_fmt(format_args!("\t\t\t\t0x{:08x?}: \"nmhd\"\n", nmhd::BOX_TYPE))?;
                v.fmt(f)?;
            }
        }
        f.write_fmt(format_args!("\n\t\t\t\t0x{:08x?}: \"dinf\"\n", dinf::BOX_TYPE))?;
        self.dinf.fmt(f)?;
        f.write_fmt(format_args!("\n\t\t\t\t0x{:08x?}: \"stbl\"\n", stbl::BOX_TYPE))?;
        self.stbl.fmt(f)?;

        Ok(())
    }
}

impl IO for minf {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();

        while 0 < r.len() {
            let mut b = Object::parse(r);

            match b.box_type {
                // vmhd: Video Media Header
                0x766d6864 => {
                    rst.mhd = MediaInformationHeader::vmhd(vmhd::parse(&mut b.payload));
                }
                // smhd: Sound Media Header
                0x736d6864 => {
                    rst.mhd = MediaInformationHeader::smhd(smhd::parse(&mut b.payload));
                }
                // hmhd: Hint Media Header
                0x686d6864 => {
                    rst.mhd = MediaInformationHeader::hmhd(hmhd::parse(&mut b.payload));
                }
                // nmhd: Null Media Header
                0x6e6d6864 => {
                    rst.mhd = MediaInformationHeader::nmhd(nmhd::parse(&mut b.payload));
                }
                // dinf: Data Information
                0x64696e66 => {
                    rst.dinf = dinf::parse(&mut b.payload);
                }
                // stbl: Sample Table
                0x7374626c => {
                    rst.stbl = stbl::parse(&mut b.payload);
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        match self.mhd.clone() {
            MediaInformationHeader::Unknown => {}
            MediaInformationHeader::vmhd(mut v) => {
                w.put(Object {
                    box_type: 0x766d6864,
                    payload: v.as_bytes(),
                }.as_bytes());
            }
            MediaInformationHeader::smhd(mut v) => {
                w.put(Object {
                    box_type: 0x736d6864,
                    payload: v.as_bytes(),
                }.as_bytes());
            }
            MediaInformationHeader::hmhd(mut v) => {
                w.put(Object {
                    box_type: 0x686d6864,
                    payload: v.as_bytes(),
                }.as_bytes());
            }
            MediaInformationHeader::nmhd(mut v) => {
                w.put(Object {
                    box_type: 0x6e6d6864,
                    payload: v.as_bytes(),
                }.as_bytes());
            }
        }

        w.put(Object {
            box_type: 0x64696e66,
            payload: self.dinf.as_bytes(),
        }.as_bytes());
        w.put(Object {
            box_type: 0x7374626c,
            payload: self.stbl.as_bytes(),
        }.as_bytes());

        w
    }
}

#[derive(Clone, PartialEq)]
enum MediaInformationHeader {
    Unknown,

    #[allow(non_camel_case_types)]
    vmhd(vmhd),
    #[allow(non_camel_case_types)]
    smhd(smhd),
    #[allow(non_camel_case_types)]
    hmhd(hmhd),
    #[allow(non_camel_case_types)]
    nmhd(nmhd),
}

impl Debug for MediaInformationHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaInformationHeader::Unknown => { Ok(()) }
            MediaInformationHeader::vmhd(v) => {
                v.fmt(f)
            }
            MediaInformationHeader::smhd(v) => {
                v.fmt(f)
            }
            MediaInformationHeader::hmhd(v) => {
                v.fmt(f)
            }
            MediaInformationHeader::nmhd(v) => {
                v.fmt(f)
            }
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq)]
pub struct vmhd {
    base: FullBox,

    graphicsmode: u16,
    opcolor: [u16; 3],
}

impl vmhd {
    pub const BOX_TYPE: u32 = 0x766d6864;
}

impl Default for vmhd {
    //! extends FullBox(‘vmhd’, version = 0, 1) {
    //!     template unsigned int(16) graphicsmode = 0; // copy, see below
    //!     template unsigned int(16)[3] opcolor = {0, 0, 0};
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 1),
            graphicsmode: 0,
            opcolor: [0_u16; 3],
        }
    }
}

impl Debug for vmhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\tgraphicsmode: {:?}", self.graphicsmode))?;
        f.write_fmt(format_args!("\n\t\t\t\t\topcolor: {:?}", self.opcolor))?;

        Ok(())
    }
}

impl IO for vmhd {
    fn parse(r: &mut BytesMut) -> Self {
        let base = FullBox::parse(r);

        Self {
            base,
            graphicsmode: r.get_u16(),
            opcolor: [
                r.get_u16(),
                r.get_u16(),
                r.get_u16(),
            ]
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u16(self.graphicsmode);
        for it in self.opcolor {
            w.put_u16(it);
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq)]
pub struct smhd {
    base: FullBox,

    balance: i16,
}

impl smhd {
    pub const BOX_TYPE: u32 = 0x736d6864;
}

impl Default for smhd {
    //! extends FullBox(‘smhd’, version = 0, 0) {
    //!     template int(16) balance = 0;
    //!     const unsigned int(16) reserved = 0;
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            balance: 0,
        }
    }
}

impl Debug for smhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\tbalance: {:?}", self.balance))?;

        Ok(())
    }
}

impl IO for smhd {
    fn parse(r: &mut BytesMut) -> Self {
        let base = FullBox::parse(r);

        let balance = r.get_i16();
        let _ = r.get_u16();

        Self {
            base,
            balance,
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_i16(self.balance);
        w.put_u16(0);

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq)]
pub struct hmhd {
    base: FullBox,

    max_pdu_size: u16,
    avg_pdu_size: u16,
    max_bitrate: u16,
    avg_bitrate: u16,
}

impl hmhd {
    pub const BOX_TYPE: u32 = 0x686d6864;
}

impl Default for hmhd {
    //! extends FullBox(’hdhd’, version = 0, flags) {unsigned int(16) maxPDUsize;
    //!     unsigned int(16) avgPDUsize;
    //!     unsigned int(32) maxbitrate;
    //!     unsigned int(32) avgbitrate;
    //!     unsigned int(32) reserved = 0;
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            max_pdu_size: 0,
            avg_pdu_size: 0,
            max_bitrate: 0,
            avg_bitrate: 0,
        }
    }
}

impl Debug for hmhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\tmax_pdu_size: {:?}", self.max_pdu_size))?;
        f.write_fmt(format_args!("\t\t\t\t\tavg_pdu_size: {:?}", self.avg_pdu_size))?;
        f.write_fmt(format_args!("\t\t\t\t\tmax_bitrate: {:?}", self.max_bitrate))?;
        f.write_fmt(format_args!("\t\t\t\t\tavg_bitrate: {:?}", self.avg_bitrate))?;

        Ok(())
    }
}

impl IO for hmhd {
    fn parse(r: &mut BytesMut) -> Self {
        let base = FullBox::parse(r);

        let max_pdu_size = r.get_u16();
        let avg_pdu_size = r.get_u16();
        let max_bitrate = r.get_u16();
        let avg_bitrate = r.get_u16();
        let _ = r.get_u32();

        Self {
            base,
            max_pdu_size,
            avg_pdu_size,
            max_bitrate,
            avg_bitrate
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u16(self.max_pdu_size);
        w.put_u16(self.avg_pdu_size);
        w.put_u16(self.max_bitrate);
        w.put_u16(self.avg_bitrate);
        w.put_u32(0);

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq)]
pub struct nmhd {
    base: FullBox,
}

impl nmhd {
    pub const BOX_TYPE: u32 = 0x6e6d6864;
}

impl Default for nmhd {
    //! extends FullBox(’nmhd’, version = 0, flags) {
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
        }
    }
}

impl Debug for nmhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\tflags: {:?}", self.base.flags))?;

        Ok(())
    }
}

impl IO for nmhd {
    fn parse(r: &mut BytesMut) -> Self {
        let base = FullBox::parse(r);

        Self {
            base,
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct dinf {
    dref: dref,
}

impl dinf {
    pub const BOX_TYPE: u32 = 0x64696e66;
}

impl Default for dinf {
    fn default() -> Self {
        Self {
            dref: Default::default(),
        }
    }
}

impl Debug for dinf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t0x{:08x?}: \"dref\"", dref::BOX_TYPE))?;
        for it in &self.dref.entries {
            match it {
                DataEntry::url_ { base, .. } => {
                    f.write_fmt(format_args!("\n\t\t\t\t\t\t0x{:08x?}: \"url_\"", 0x75726c20))?;
                    f.write_fmt(format_args!("\n\t\t\t\t\t\t\tflags: {:?}", base.flags))?;
                }
            }
        }

        Ok(())
    }
}

impl IO for dinf {
    fn parse(r: &mut BytesMut) -> Self {
        let mut b = Object::parse(r);

        Self {
            dref: dref::parse(&mut b.payload),
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: {
                let mut v = 0_u32;
                for it in "dinf".as_bytes() {
                    v = (v << 8) | (*it as u32);
                }
                v
            },
            payload: self.dref.as_bytes(),
        }.as_bytes());

        w
    }
}

#[derive(PartialEq)]
pub enum DataEntry {
    #[allow(non_camel_case_types)]
    url_ {
        base: FullBox,
        location: String,
    }
}

impl IO for DataEntry {
    fn parse(r: &mut BytesMut) -> Self {
        let mut b = Object::parse(r);
        match b.box_type {
            // url : URL
            0x75726c20 => {
                let base = FullBox::parse(&mut b.payload);

                DataEntry::url_ {
                    base,
                    location: std::str::from_utf8(b.payload.split_to(b.payload.len()).chunk()).unwrap().to_owned(),
                }
            }
            _ => {
                unimplemented!()
            }
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(match self {
            DataEntry::url_ { base, location } => {
                Object {
                    box_type: 0x75726c20,
                    payload: {
                        let mut w = BytesMut::new();

                        w.put(base.as_bytes());

                        w.put(location.as_bytes());

                        w
                    }
                }
            }
        }.as_bytes());

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct dref {
    base: FullBox,

    entries: Vec<DataEntry>,
}

impl dref {
    pub const BOX_TYPE: u32 = 0x64726566;
}

impl Default for dref {
    //! extends FullBox(‘dref’, version = 0, 0) {
    //!     unsigned int(32) entry_count;
    //!     for (i=1; i • entry_count; i++) {
    //!         DataEntryBox(entry_version, entry_flags) data_entry;
    //!     }
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            entries: vec![],
        }
    }
}

impl IO for dref {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            entries: vec![]
        };

        let entry_count = r.get_u32();

        for _ in 0..entry_count {
            rst.entries.push(DataEntry::parse(r));
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(self.entries.len() as u32);

        for it in self.entries.iter_mut() {
            w.put(it.as_bytes());
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct stbl {
    stsd: stsd,
    stts: stts,
    stsc: stsc,
    stsz: stsz,
    stco: stco,
}

impl Default for stbl {
    fn default() -> Self {
        Self {
            stsd: Default::default(),
            stts: Default::default(),
            stsc: Default::default(),
            stsz: Default::default(),
            stco: Default::default(),
        }
    }
}

impl stbl {
    pub const BOX_TYPE: u32 = 0x7374626c;
}

impl Debug for stbl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t0x{:08x?}: \"stsd\"", stsd::BOX_TYPE))?;
        f.write_fmt(format_args!("\n{:?}", self.stsd))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t0x{:08x?}: \"stts\"", stts::BOX_TYPE))?;
        f.write_fmt(format_args!("\n{:?}", self.stts))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t0x{:08x?}: \"stsz\"", stsz::BOX_TYPE))?;
        f.write_fmt(format_args!("\n{:?}", self.stsz))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t0x{:08x?}: \"stsc\"", stsc::BOX_TYPE))?;
        f.write_fmt(format_args!("\n{:?}", self.stsc))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t0x{:08x?}: \"stco\"", stco::BOX_TYPE))?;
        f.write_fmt(format_args!("\n{:?}", self.stco))?;

        Ok(())
    }
}

impl IO for stbl {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();

        while 0 < r.len() {
            let mut b = Object::parse(r);

            match b.box_type {
                // stsd: Sample Description
                0x73747364 => {
                    rst.stsd = stsd::parse(&mut b.payload);
                }
                // stts: Decoding Time to Sample
                0x73747473 => {
                    rst.stts = stts::parse(&mut b.payload);
                }
                // stsc: Sample To Chunk
                0x73747363 => {
                    rst.stsc = stsc::parse(&mut b.payload);
                }
                // stsz: Sample Size
                0x7374737a => {
                    rst.stsz = stsz::parse(&mut b.payload);
                }
                // stco: Chunk Offset
                0x7374636f => {
                    rst.stco = stco::parse(&mut b.payload);
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: 0x73747364,
            payload: self.stsd.as_bytes(),
        }.as_bytes());
        w.put(Object {
            box_type: 0x73747473,
            payload: self.stts.as_bytes(),
        }.as_bytes());
        w.put(Object {
            box_type: 0x73747363,
            payload: self.stsc.as_bytes(),
        }.as_bytes());
        w.put(Object {
            box_type: 0x7374737a,
            payload: self.stsz.as_bytes(),
        }.as_bytes());
        w.put(Object {
            box_type: 0x7374636f,
            payload: self.stco.as_bytes(),
        }.as_bytes());

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct stsd {
    base: FullBox,

    entries: Vec<SampleEntry>,
}

impl stsd {
    pub const BOX_TYPE: u32 = 0x73747364;
}

impl Default for stsd {
    //! extends FullBox('stsd', 0, 0){
    //!     int i ;
    //!     unsigned int(32) entry_count;
    //!     for (i = 1 ; i u entry_count ; i++){
    //!         switch (handler_type){
    //!             case ‘soun’: // for audio tracks
    //!                 AudioSampleEntry();
    //!                 break;
    //!             case ‘vide’: // for video tracks
    //!                 VisualSampleEntry();
    //!                 break;
    //!             case ‘hint’: // Hint track
    //!                 HintSampleEntry();
    //!                 break;
    //!         }
    //!     }
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            entries: vec![],
        }
    }
}

impl Debug for stsd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t\tentry_count: {:?}", self.entries.len()))?;
        for it in &self.entries {
            f.write_fmt(format_args!("\n{:?}", it))?;
        }

        Ok(())
    }
}

impl IO for stsd {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            entries: vec![],
        };

        let entry_count = r.get_u32();
        for _ in 0..entry_count {
            rst.entries.push(SampleEntry::parse(r));
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(self.entries.len() as u32);

        for it in self.entries.iter_mut() {
            w.put(Object {
                box_type: it.get_handler_type(),
                payload: it.as_bytes(),
            }.as_bytes());
        }

        w
    }
}

#[derive(Clone, PartialEq)]
pub enum SampleEntry {
    Base {
        handler_type: u32,
        data_reference_index: u16,
    },
    Visual {
        base: std::boxed::Box<SampleEntry>,

        width: u16,
        height: u16,
        horiz_resolution: u32,
        vert_resolution: u32,
        frame_count: u16,
        compressor_name: String,
        depth: u16,
    },
    Audio {
        base: std::boxed::Box<SampleEntry>,

        channel_count: u16,
        sample_size: u16,
        sample_rate: u32,
    },
    #[allow(non_camel_case_types)]
    #[allow(non_snake_case)]
    avc1 {
        base: std::boxed::Box<SampleEntry>,

        avcC: avcC,
    },
}

impl Debug for SampleEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleEntry::Base {
                handler_type,
                data_reference_index,
            } => {
                f.write_fmt(format_args!("\t\t\t\t\t\t\tformat: 0x{:08x?}: {:?}", handler_type, std::str::from_utf8(&handler_type.to_be_bytes()).unwrap_or("")))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tdata_reference_index: {:?}", data_reference_index))?;
            }
            SampleEntry::Visual {
                base,
                width,
                height,
                horiz_resolution,
                vert_resolution,
                frame_count,
                compressor_name,
                depth ,
            } => {
                base.fmt(f)?;

                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\twidth: {:?}", width))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\theight: {:?}", height))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\thoriz_resolution: 0x{:08x?}", horiz_resolution))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tvert_resolution: 0x{:08x?}", vert_resolution))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tframe_count: {:?}", frame_count))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tcompressor_name: {:?}", compressor_name))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tdepth: 0x{:04x?}", depth))?;
            }
            SampleEntry::Audio {
                base,
                channel_count,
                sample_size,
                sample_rate,
            } => {
                base.fmt(f)?;

                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tchannel_count: {:?}", channel_count))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tsample_size: {:?}", sample_size))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tsample_rate: {:?}", sample_rate))?;
            }
            SampleEntry::avc1 {
                base,
                avcC,
            } => {
                base.fmt(f)?;

                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tavcC:\n{:?}", avcC))?;

            }
        }

        Ok(())
    }
}

impl IO for SampleEntry {
    fn parse(r: &mut BytesMut) -> Self {
        let mut b = Object::parse(r);
        let handler_type = b.box_type;
        let _ = b.payload.split_to(6);
        let data_reference_index = b.payload.get_u16();

        let base = SampleEntry::Base {
            handler_type,
            data_reference_index,
        };

        match handler_type {
            // avc1
            0x61766331 => {
                let _ = b.payload.get_u16();
                let _ = b.payload.get_u16();
                let _ = b.payload.split_to(12);
                let width = b.payload.get_u16();
                let height = b.payload.get_u16();
                let horiz_resolution = b.payload.get_u32();
                let vert_resolution = b.payload.get_u32();
                let _ = b.payload.get_u32();
                let frame_count = b.payload.get_u16();
                let compressor_name = {
                    let len = min(31, b.payload.get_u8());
                    let mut rst = String::with_capacity(len as usize);

                    rst.push_str(std::str::from_utf8(b.payload.split_to(len as usize).chunk()).unwrap());
                    if 31 > len {
                        let _ = b.payload.split_to((31 - len) as usize);
                    }

                    rst
                };
                let depth = b.payload.get_u16();
                let _ = b.payload.get_u16();

                let vide = SampleEntry::Visual {
                    base: std::boxed::Box::new(base),
                    width,
                    height,
                    horiz_resolution,
                    vert_resolution,
                    frame_count,
                    compressor_name,
                    depth,
                };

                while 0 < b.payload.len() {
                    let mut b = Object::parse(&mut b.payload);

                    match b.box_type {
                        // avcC
                        0x61766343 => {
                            return SampleEntry::avc1 {
                                base: std::boxed::Box::new(vide),
                                avcC: avcC::parse(&mut b.payload),
                            }
                        }
                        _ => {
                        }
                    }
                }

                vide
            }
            // mp4a
            0x6d703461 => {
                let _ = b.payload.get_u64();
                let channel_count = b.payload.get_u16();
                let sample_size = b.payload.get_u16();
                let _ = b.payload.get_u32();
                let sample_rate = b.payload.get_u32();
                SampleEntry::Audio {
                    base: std::boxed::Box::new(base),
                    channel_count,
                    sample_size,
                    sample_rate,
                }
            }
            _ => {
                base
            }
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        match self {
            SampleEntry::Base { data_reference_index, .. } => {
                w.put_bytes(0, 6);
                w.put_u16(*data_reference_index);
            }
            SampleEntry::Visual {
                base,
                width,
                height,
                horiz_resolution,
                vert_resolution,
                frame_count,
                compressor_name,
                depth,
            } => {
                w.put(base.as_bytes());

                w.put_u16(0);
                w.put_u16(0);
                w.put_bytes(0, 12);
                w.put_u16(*width);
                w.put_u16(*height);
                w.put_u32(*horiz_resolution);
                w.put_u32(*vert_resolution);
                w.put_u32(0);
                w.put_u16(*frame_count);
                {
                    let len = min(31, compressor_name.len()) as u8;
                    w.put_u8(len);
                    w.put_slice(&compressor_name.as_bytes()[..len as usize]);
                    if 31 > len {
                        w.put_bytes(0, (31 - len) as usize);
                    }
                }
                w.put_u16(*depth);
                w.put_u16(0xffff);
            }
            SampleEntry::Audio {
                base,
                channel_count,
                sample_size,
                sample_rate,
            } => {
                w.put(base.as_bytes());

                w.put_u64(0);
                w.put_u16(*channel_count);
                w.put_u16(*sample_size);
                w.put_u32(0);
                w.put_u32(*sample_rate);
            }
            SampleEntry::avc1 {
                base,
                avcC,
            } => {
                w.put(base.as_bytes());

                w.put(Object {
                    box_type: 0x61766343,
                    payload: avcC.as_bytes(),
                }.as_bytes());
            }
        }

        w
    }
}

impl SampleEntry {
    fn get_handler_type(&self) -> u32 {
        match self {
            SampleEntry::Base { handler_type, .. } => {
                *handler_type
            }
            SampleEntry::Visual { base, .. } => {
                base.get_handler_type()
            }
            SampleEntry::Audio { base, .. } => {
                base.get_handler_type()
            }
            SampleEntry::avc1 { base, .. } => {
                base.get_handler_type()
            }
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct stts {
    base: FullBox,

    entries: Vec<(u32, u32)>,
}

impl stts {
    pub const BOX_TYPE: u32 = 0x73747473;
}

impl Default for stts {
    //! extends FullBox(’stts’, version = 0, 0) {
    //!     unsigned int(32) entry_count;
    //!     int i;
    //!     for (i=0; i < entry_count; i++) {
    //!         unsigned int(32) sample_count;
    //!         unsigned int(32) sample_delta;
    //!     }
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            entries: vec![],
        }
    }
}

impl Debug for stts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t\tentry_count: {:?}", self.entries.len()))?;
        for (sample_count, sample_delta) in &self.entries {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\tsample_count: {:?}", sample_count))?;
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\tsample_delta: {:?}", sample_delta))?;
        }

        Ok(())
    }
}

impl IO for stts {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            entries: vec![]
        };

        let entry_count = r.get_u32();
        for _ in 0..entry_count {
            rst.entries.push((r.get_u32(), r.get_u32()))
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(self.entries.len() as u32);

        for (count, delta) in &self.entries {
            w.put_u32(*count);
            w.put_u32(*delta);
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct stsc {
    base: FullBox,

    entries: Vec<(u32, u32, u32)>,
}

impl stsc {
    pub const BOX_TYPE: u32 = 0x73747363;
}

impl Default for stsc {
    //! extends FullBox(‘stsc’, version = 0, 0) {
    //!     unsigned int(32) entry_count;
    //!     for (i=1; i u entry_count; i++) {
    //!         unsigned int(32) first_chunk;
    //!         unsigned int(32) samples_per_chunk;
    //!         unsigned int(32) sample_description_index;
    //!     }
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            entries: vec![],
        }
    }
}

impl Debug for stsc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t\tentry_count: {:?}", self.entries.len()))?;
        for (first_chunk, samples_per_chunk, sample_description_index) in &self.entries {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\tfirst_chunk: {:?}", first_chunk))?;
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\tsamples_per_chunk: {:?}", samples_per_chunk))?;
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\tsample_description_index: {:?}", sample_description_index))?;
        }

        Ok(())
    }
}

impl IO for stsc {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            entries: vec![]
        };

        let entry_count = r.get_u32();
        for _ in 0..entry_count {
            rst.entries.push((r.get_u32(), r.get_u32(), r.get_u32()))
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(self.entries.len() as u32);

        for (first_chunk, samples_per_chunk, sample_description_index) in &self.entries {
            w.put_u32(*first_chunk);
            w.put_u32(*samples_per_chunk);
            w.put_u32(*sample_description_index);
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct stsz {
    base: FullBox,

    sample_size: u32,
    entries: Vec<u32>,
}

impl stsz {
    pub const BOX_TYPE: u32 = 0x7374737a;
}

impl Default for stsz {
    //! extends FullBox(‘stsz’, version = 0, 0) {
    //!     unsigned int(32) sample_size;
    //!     unsigned int(32) sample_count;
    //!     if (sample_size==0) {
    //!         for (i=1; i u sample_count; i++) {
    //!             unsigned int(32) entry_size;
    //!         }
    //!     }
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            sample_size: 0,
            entries: vec![],
        }
    }
}

impl Debug for stsz {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t\tsample_size: {:?}", self.sample_size))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\tsample_count: {:?}", self.entries.len()))?;
        if 0 == self.sample_size && 0 < self.entries.len() {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\tentry: {:?}", self.entries))?;
        }

        Ok(())
    }
}

impl IO for stsz {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            sample_size: r.get_u32(),
            entries: vec![]
        };

        if 0 == rst.sample_size {
            let sample_count = r.get_u32();
            for _ in 0..sample_count {
                rst.entries.push(r.get_u32())
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(self.sample_size);
        w.put_u32(self.entries.len() as u32);

        if 0 == self.sample_size {
            for entry_size in &self.entries {
                w.put_u32(*entry_size);
            }
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct stco {
    base: FullBox,

    entries: Vec<u32>,
}

impl stco {
    pub const BOX_TYPE: u32 = 0x7374636f;
}

impl Default for stco {
    //! extends FullBox(‘stco’, version = 0, 0) {
    //!     unsigned int(32) entry_count;
    //!     for (i=1; i u entry_count; i++) {
    //!         unsigned int(32) chunk_offset;
    //!     }
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            entries: vec![],
        }
    }
}

impl Debug for stco {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t\tentry_count: {:?}", self.entries.len()))?;
        for chunk_offset in &self.entries {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\tchunk_offset: {:?}", chunk_offset))?;
        }

        Ok(())
    }
}

impl IO for stco {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            entries: vec![]
        };

        let entry_count = r.get_u32();
        for _ in 0..entry_count {
            rst.entries.push(r.get_u32())
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(self.entries.len() as u32);

        for chunk_offset in &self.entries {
            w.put_u32(*chunk_offset);
        }

        w
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use crate::{FullBox, IO, Object};
    use crate::moov::{DataEntry, dinf, dref, hdlr, mdhd, mdia, MediaInformationHeader, minf, moov, mvhd, SampleEntry, smhd, stbl, stco, stsc, stsd, stsz, stts, tkhd, tkhd_flags, trak, vmhd};
    use crate::moov::avc::avcC;

    #[test]
    fn chk_moov() {
        let mut b = moov {
            mvhd: mvhd {
                creation_time: 0,
                modification_time: 0,
                timescale: 1000,
                duration: 0,
                rate: 0x00010000,
                volume: 0x0100,
                matrix: [0x00010000,0,0,0,0x00010000,0,0,0,0x40000000],
                next_track_id: 3,
            },
            traks: vec![
                trak {
                    tkhd: tkhd {
                        base: FullBox::new(0, tkhd_flags::TRACK_ENABLED | tkhd_flags::TRACK_IN_MOVIE | tkhd_flags::TRACK_IN_PREVIEW),
                        creation_time: 0,
                        modification_time: 3503872213,
                        track_id: 1,
                        duration: 0,
                        layer: 0,
                        alternate_group: 0,
                        volume: 0x0100,
                        matrix: [0x00010000,0,0,0,0x00010000,0,0,0,0x40000000],
                        width: 26214400,
                        height: 19660800,
                    },
                    mdia: mdia {
                        mdhd: mdhd {
                            base: FullBox::new(0, 0),
                            creation_time: 0,
                            modification_time: 0,
                            timescale: 90000,
                            duration: 0,
                            language: {
                                let mut rst = 0_u16;
                                for c in "und".as_bytes() {
                                    rst = (rst << 5) | (0b11111 & (c - 0x60)) as u16;
                                }
                                rst
                            },
                        },
                        hdlr: hdlr {
                            base: FullBox::new(0, 0),
                            handler_type: 0x76696465,
                            name: "VideoHandler\u{0}".to_owned(),
                        },
                        minf: minf {
                            mhd: MediaInformationHeader::vmhd(vmhd {
                                base: FullBox::new(0, 0),
                                graphicsmode: 0,
                                opcolor: [0, 0, 0],
                            }),
                            dinf: dinf {
                                dref: dref {
                                    base: FullBox::new(0, 0),
                                    entries: vec![
                                        DataEntry::url_ {
                                            base: FullBox::new(0, 0x000001),
                                            location: "".to_owned(),
                                        }
                                    ]
                                }
                            },
                            stbl: stbl {
                                stsd: stsd {
                                    base: FullBox::new(0, 0),
                                    entries: vec![
                                        SampleEntry::avc1 {
                                            base: Box::new(SampleEntry::Visual {
                                                base: Box::new(SampleEntry::Base {
                                                    handler_type: 0x61766331,
                                                    data_reference_index: 1,
                                                }),
                                                width: 400,
                                                height: 300,
                                                horiz_resolution: 0x00480000,
                                                vert_resolution: 0x00480000,
                                                frame_count: 1,
                                                compressor_name: "".to_owned(),
                                                depth: 24,
                                            }),
                                            avcC: avcC {
                                                configuration_version: 1,
                                                profile_indication: 77,
                                                profile_compatibility: 64,
                                                level_indication: 21,
                                                length_size_minus_one: 3,
                                                sps: vec![
                                                    {
                                                        let mut rst = BytesMut::new();
                                                        rst.put(&b"'M@\x15\xa9\x182\x13\xfd\xe0\rA\x80A\xad\xb0\xad{\xdf\x01"[..]);
                                                        rst
                                                    }
                                                ],
                                                pps: vec![
                                                    {
                                                        let mut rst = BytesMut::new();
                                                        rst.put(&b"(\xde\t\x88"[..]);
                                                        rst
                                                    }
                                                ],
                                            },
                                        }
                                    ],
                                },
                                stts: stts {
                                    base: FullBox::new(0, 0),
                                    entries: vec![],
                                },
                                stsc: stsc {
                                    base: FullBox::new(0, 0),
                                    entries: vec![],
                                },
                                stsz: stsz {
                                    base: FullBox::new(0, 0),
                                    sample_size: 0,
                                    entries: vec![],
                                },
                                stco: stco {
                                    base: FullBox::new(0, 0),
                                    entries: vec![],
                                },
                            },
                        },
                    },
                },
                trak {
                    tkhd: tkhd {
                        base: FullBox::new(0, 0),
                        creation_time: 0,
                        modification_time: 3503872213,
                        track_id: 2,
                        duration: 0,
                        layer: 0,
                        alternate_group: 1,
                        volume: 0x0100,
                        matrix: [0x00010000,0,0,0,0x00010000,0,0,0,0x40000000],
                        width: 0,
                        height: 0,
                    },
                    mdia: mdia {
                        mdhd: mdhd {
                            base: FullBox::new(0, 0),
                            creation_time: 0,
                            modification_time: 0,
                            timescale: 22050,
                            duration: 0,
                            language: {
                                let mut rst = 0_u16;
                                for c in "und".as_bytes() {
                                    rst = (rst << 5) | (0b11111 & (c - 0x60)) as u16;
                                }
                                rst
                            },
                        },
                        hdlr: hdlr {
                            base: FullBox::new(0, 0),
                            handler_type: 0x736f756e,
                            name: "SoundHandler\u{0}".to_owned(),
                        },
                        minf: minf {
                            mhd: MediaInformationHeader::smhd(smhd {
                                base: FullBox::new(0, 0),
                                balance: 0,
                            }),
                            dinf: dinf {
                                dref: dref {
                                    base: FullBox::new(0, 0),
                                    entries: vec![
                                        DataEntry::url_ {
                                            base: FullBox::new(0, 0x000001),
                                            location: "".to_owned(),
                                        }
                                    ]
                                }
                            },
                            stbl: stbl {
                                stsd: stsd {
                                    base: FullBox::new(0, 0),
                                    entries: vec![
                                        SampleEntry::Audio {
                                            base: Box::new(SampleEntry::Base {
                                                handler_type: 0x6d703461,
                                                data_reference_index: 1,
                                            }),
                                            channel_count: 2,
                                            sample_size: 16,
                                            sample_rate: 1445068800,
                                        }
                                    ]
                                },
                                stts: stts {
                                    base: FullBox::new(0, 0),
                                    entries: vec![]
                                },
                                stsc: stsc {
                                    base: FullBox::new(0, 0),
                                    entries: vec![]
                                },
                                stsz: stsz {
                                    base: FullBox::new(0, 0),
                                    sample_size: 0,
                                    entries: vec![],
                                },
                                stco: stco {
                                    base: FullBox::new(0, 0),
                                    entries: vec![],
                                },
                            },
                        },
                    },
                },
            ],
        };
        let mut obj = Object::parse(&mut Object {
            box_type: moov::BOX_TYPE,
            payload: b.as_bytes(),
        }.as_bytes());

        assert_eq!(moov::BOX_TYPE, obj.box_type);
        assert_eq!(b, moov::parse(&mut obj.payload));
    }
}
