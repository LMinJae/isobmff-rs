use std::fmt::{Debug, Formatter};
use bytes::{Buf, BufMut, BytesMut};

pub trait IO {
    fn parse(r: &mut BytesMut) -> Self;
    fn as_bytes(&mut self) -> BytesMut;
}

pub struct Box {
    pub box_type: u32,
    pub payload: BytesMut,
}

impl IO for Box {
    fn parse(r: &mut BytesMut) -> Self {
        let mut size = r.get_u32() as u64;
        let box_type = r.get_u32();
        if 1 == size {
            size = r.get_u64() - 8;
        }
        if 0 == size {
            Self {
                box_type,
                payload: r.split_to(r.len()),
            }
        } else {
            Self {
                box_type,
                payload: r.split_to((size as usize) - 8),
            }
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        let size = 4 + self.payload.len();
        /*  */ if (u32::MAX as usize) < size {
            w.put_u32(1);
        } else if (u64::MAX as usize) < size {
            w.put_u32(0);
        }
        w.put_u32(self.box_type);
        /*  */ if (u32::MAX as usize) < size && size <= (u64::MAX as usize) {
            w.put_u64(size as u64);
        }
        w.put(self.payload.chunk());

        w
    }
}

#[derive(Clone)]
pub struct FullBox {
    version: u8,
    flags: u32,
}

impl IO for FullBox {
    fn parse(r: &mut BytesMut) -> Self {
        let t = r.get_u32();

        Self {
            version: (t >> 24) as u8,
            flags: t & 0x00FFFFFF,
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u32((self.version as u32) << 24 | self.flags);

        w
    }
}

#[allow(non_camel_case_types)]
pub struct ftyp {
    major_brand: u32,
    minor_version: u32,
    compatible_brands: Vec<u32>,
}

impl Debug for ftyp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\tmajor_brand:"))?;
        if let Ok(str) = std::str::from_utf8(&self.major_brand.to_be_bytes()) {
            f.write_fmt(format_args!(" {:?}", str))?;
        } else {
            f.write_fmt(format_args!(" 0x{:08?}", self.major_brand))?;
        }
        f.write_fmt(format_args!("\n\tminor_version: {:?}", self.minor_version))?;
        f.write_fmt(format_args!("\n\tcompatible_brands:"))?;
        for it in &self.compatible_brands {
            if let Ok(str) = std::str::from_utf8(&it.to_be_bytes()) {
                f.write_fmt(format_args!(" {:?}", str))?;
            } else {
                f.write_fmt(format_args!(" 0x{:08?}", it))?;
            }
        }

        Ok(())
    }
}

impl IO for ftyp {
    fn parse(r: &mut BytesMut) -> Self {
        Self {
            major_brand: r.get_u32(),
            minor_version: r.get_u32(),
            compatible_brands: {
                let mut v = Vec::<u32>::with_capacity(r.len() / 4);
                while 0 < r.len() {
                    v.push(r.get_u32())
                }
                v
            }
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u32(self.major_brand);
        w.put_u32(self.minor_version);
        for it in &self.compatible_brands {
            w.put_u32(*it);
        }

        w
    }
}

#[allow(non_camel_case_types)]
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

impl Debug for mvhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\tcreation_time: {:?}", self.creation_time))?;
        f.write_fmt(format_args!("\n\t\tmodification_time: {:?}", self.modification_time))?;
        f.write_fmt(format_args!("\n\t\ttimescale: {:?}", self.timescale))?;
        f.write_fmt(format_args!("\n\t\tduration: {:?}", self.duration))?;
        f.write_fmt(format_args!("\n\t\trate: {:?}", self.rate))?;
        f.write_fmt(format_args!("\n\t\tvolume: {:?}", self.volume))?;
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

impl Default for mvhd {
    fn default() -> Self {
        Self {
            creation_time: 0,
            modification_time: 0,
            timescale: 0,
            duration: 0,
            rate: 0x00010000,
            volume: 0x0100,
            matrix: [0x00010000,0,0,0,0x00010000,0,0,0,0x40000000],
            next_track_id: 0
        }
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

impl Debug for tkhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\tcreation_time: {:?}", self.creation_time))?;
        f.write_fmt(format_args!("\n\t\t\tmodification_time: {:?}", self.modification_time))?;
        f.write_fmt(format_args!("\n\t\t\ttrack_id: {:?}", self.track_id))?;
        f.write_fmt(format_args!("\n\t\t\tduration: {:?}", self.duration))?;

        f.write_fmt(format_args!("\n\t\t\tlayer: {:?}", self.layer))?;
        f.write_fmt(format_args!("\n\t\t\talternate_group: {:?}", self.alternate_group))?;
        f.write_fmt(format_args!("\n\t\t\tvolume: {:?}", self.volume))?;
        f.write_fmt(format_args!("\n\t\t\twidth: {:?}", self.width))?;
        f.write_fmt(format_args!("\n\t\t\theight: {:?}", self.height))?;

        Ok(())
    }
}

impl Default for tkhd {
    fn default() -> Self {
        Self {
            base: FullBox { version: 0, flags: 0 },
            creation_time: 0,
            modification_time: 0,
            track_id: 0,
            duration: 0,
            layer: 0,
            alternate_group: 0,
            volume: 0,
            matrix: [0x00010000,0,0,0,0x00010000,0,0,0,0x40000000],
            width: 0,
            height: 0
        }
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
pub struct mdhd {
    base: FullBox,

    creation_time: u64,
    modification_time: u64,
    timescale: u32,
    duration: u64,
    language: u16,
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
        let version = r.get_u8();
        let _flags = r.split_to(3);

        let mut rst = Self {
            base: FullBox { version, flags: 0 },
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
pub struct hdlr {
    base: FullBox,

    handler_type: u32,
    name: String,
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
pub struct minf {
    mhd: MediaInformationHeader,
    dinf: dinf,
    stbl: stbl,
}

impl Debug for minf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.mhd {
            MediaInformationHeader::Unknown => {}
            MediaInformationHeader::vmhd(v) => {
                f.write_fmt(format_args!("\t\t\t\t0x766d6864: \"vmhd\"\n"))?;
                v.fmt(f)?;
            }
            MediaInformationHeader::smhd(v) => {
                f.write_fmt(format_args!("\t\t\t\t0x736d6864: \"smhd\"\n"))?;
                v.fmt(f)?;
            }
            MediaInformationHeader::hmhd(v) => {
                f.write_fmt(format_args!("\t\t\t\t0x686d6864: \"hmhd\"\n"))?;
                v.fmt(f)?;
            }
            MediaInformationHeader::nmhd(v) => {
                f.write_fmt(format_args!("\t\t\t\t0x6e6d6864: \"nmhd\"\n"))?;
                v.fmt(f)?;
            }
        }
        f.write_fmt(format_args!("\n\t\t\t\t0x64696e66: \"dinf\"\n"))?;
        self.dinf.fmt(f)?;
        f.write_fmt(format_args!("\n\t\t\t\t0x7374626c: \"stbl\"\n"))?;
        self.stbl.fmt(f)?;

        Ok(())
    }
}

impl IO for minf {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            mhd: MediaInformationHeader::Unknown,
            dinf: dinf { dref: dref { base: FullBox { version: 0, flags: 0 }, entries: vec![] } },
            stbl: stbl {
                stsd: stsd { base: FullBox { version: 0, flags: 0 }, entries: vec![] },
                stts: stts { base: FullBox { version: 0, flags: 0 }, entries: vec![] },
                stsc: stsc { base: FullBox { version: 0, flags: 0 }, entries: vec![] },
                stsz: stsz {
                    base: FullBox { version: 0, flags: 0 },
                    sample_size: 0,
                    entries: vec![]
                },
                stco: stco { base: FullBox { version: 0, flags: 0 }, entries: vec![] },
            },
        };
        
        while 0 < r.len() {
            let mut b = Box::parse(r);

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
                w.put(Box {
                    box_type: 0x766d6864,
                    payload: v.as_bytes(),
                }.as_bytes());
            }
            MediaInformationHeader::smhd(mut v) => {
                w.put(Box {
                    box_type: 0x736d6864,
                    payload: v.as_bytes(),
                }.as_bytes());
            }
            MediaInformationHeader::hmhd(mut v) => {
                w.put(Box {
                    box_type: 0x686d6864,
                    payload: v.as_bytes(),
                }.as_bytes());
            }
            MediaInformationHeader::nmhd(mut v) => {
                w.put(Box {
                    box_type: 0x6e6d6864,
                    payload: v.as_bytes(),
                }.as_bytes());
            }
        }

        w.put(Box {
            box_type: 0x64696e66,
            payload: self.dinf.as_bytes(),
        }.as_bytes());
        w.put(Box {
            box_type: 0x7374626c,
            payload: self.stbl.as_bytes(),
        }.as_bytes());

        w
    }
}

#[derive(Clone)]
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
#[derive(Clone)]
pub struct vmhd {
    base: FullBox,

    graphicsmode: u16,
    opcolor: [u16; 3],
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
#[derive(Clone)]
pub struct smhd {
    base: FullBox,

    balance: i16,
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

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct hmhd {
    base: FullBox,

    max_pdu_size: u16,
    avg_pdu_size: u16,
    max_bitrate: u16,
    avg_bitrate: u16,
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
#[derive(Clone)]
pub struct nmhd {
    base: FullBox,
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
pub struct dinf {
    dref: dref,
}

impl Debug for dinf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t0x64726566: \"dref\""))?;
        for it in &self.dref.entries {
            match it {
                DataEntry::url_ { base, .. } => {
                    f.write_fmt(format_args!("\n\t\t\t\t\t\tflags: {:?}", base.flags))?;
                }
            }
        }

        Ok(())
    }
}

impl IO for dinf {
    fn parse(r: &mut BytesMut) -> Self {
        let mut b = Box::parse(r);

        let rst = Self {
            dref: dref::parse(&mut b.payload)
        };

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Box {
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

pub enum DataEntry {
    #[allow(non_camel_case_types)]
    url_ {
        base: FullBox,
        location: String,
    }
}

impl IO for DataEntry {
    fn parse(r: &mut BytesMut) -> Self {
        let mut b = Box::parse(r);
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

        match self {
            DataEntry::url_ { base, location } => {
                w.put(base.as_bytes());

                w.put(location.as_bytes());
            }
        }

        w
    }
}

#[allow(non_camel_case_types)]
pub struct dref {
    base: FullBox,

    entries: Vec<DataEntry>,
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
pub struct stbl {
    stsd: stsd,
    stts: stts,
    stsc: stsc,
    stsz: stsz,
    stco: stco,
}

impl Debug for stbl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t0x73747364: \"stsd\""))?;
        f.write_fmt(format_args!("\n{:?}", self.stsd))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t0x73747473: \"stts\""))?;
        f.write_fmt(format_args!("\n{:?}", self.stts))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t0x7374737a: \"stsz\""))?;
        f.write_fmt(format_args!("\n{:?}", self.stsz))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t0x73747363: \"stsc\""))?;
        f.write_fmt(format_args!("\n{:?}", self.stsc))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t0x7374636f: \"stco\""))?;
        f.write_fmt(format_args!("\n{:?}", self.stco))?;

        Ok(())
    }
}

impl IO for stbl {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            stsd: stsd { base: FullBox { version: 0, flags: 0 }, entries: vec![] },
            stts: stts { base: FullBox { version: 0, flags: 0 }, entries: vec![] },
            stsc: stsc { base: FullBox { version: 0, flags: 0 }, entries: vec![] },
            stsz: stsz { base: FullBox { version: 0, flags: 0 }, sample_size: 0, entries: vec![] },
            stco: stco { base: FullBox { version: 0, flags: 0 }, entries: vec![] },
        };

        while 0 < r.len() {
            let mut b = Box::parse(r);

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

        w.put(Box {
            box_type: 0x73747364,
            payload: self.stsd.as_bytes(),
        }.as_bytes());
        w.put(Box {
            box_type: 0x73747473,
            payload: self.stts.as_bytes(),
        }.as_bytes());
        w.put(Box {
            box_type: 0x73747363,
            payload: self.stsc.as_bytes(),
        }.as_bytes());
        w.put(Box {
            box_type: 0x7374737a,
            payload: self.stsz.as_bytes(),
        }.as_bytes());
        w.put(Box {
            box_type: 0x7374636f,
            payload: self.stco.as_bytes(),
        }.as_bytes());

        w
    }
}

#[allow(non_camel_case_types)]
pub struct stsd {
    base: FullBox,

    entries: Vec<SampleEntry>,
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
            w.put(Box {
                box_type: it.get_handler_type(),
                payload: it.as_bytes()
            }.as_bytes());
        }

        w
    }
}

#[derive(Clone)]
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
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\thoriz_resolution: {:?}", horiz_resolution))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tvert_resolution: {:?}", vert_resolution))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tframe_count: {:?}", frame_count))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tcompressor_name: {:?}", compressor_name))?;
                f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\tdepth: {:?}", depth))?;
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

                f.write_fmt(format_args!("\n{:?}", avcC))?;

            }
        }

        Ok(())
    }
}

impl IO for SampleEntry {
    fn parse(r: &mut BytesMut) -> Self {
        let mut b = Box::parse(r);
        let _ = b.payload.split_to(6);
        let handler_type = b.box_type;
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
                let compressor_name = std::str::from_utf8(b.payload.split_to(32).chunk()).unwrap_or("").to_owned();
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
                    let mut b = Box::parse(&mut b.payload);

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
                for _ in 0..5 {
                    w.put_u8(0);
                }
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
                for _ in 0..12 {
                    w.put_u8(0);
                }
                w.put_u16(*width);
                w.put_u16(*height);
                w.put_u32(*horiz_resolution);
                w.put_u32(*vert_resolution);
                w.put_u32(0);
                w.put_u16(*frame_count);
                w.put(&compressor_name.as_bytes()[..32]);
                w.put_u16(*depth);
                w.put_u16(0);
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

                w.put(avcC.as_bytes());
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
#[derive(Clone)]
pub struct avcC {
    configuration_version: u8,
    profile_indication: u8,
    profile_compatibility: u8,
    level_indication: u8,
    length_size_minus_one: u8,
    sps: Vec<BytesMut>,
    pps: Vec<BytesMut>,
}

impl Debug for avcC {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t\t\t\t\t\tconfiguration_version: {:?}", self.configuration_version))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tprofile_indication: {:?}", self.profile_indication))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tprofile_compatibility: {:?}", self.profile_compatibility))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tlevel_indication: {:?}", self.level_indication))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tlength_size_minus_one: {:?}", self.length_size_minus_one))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tnb_sps: {:?}", self.sps.len()))?;
        for it in &self.sps {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\t\t{:x?}", it))?;
        }
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tnb_pps: {:?}", self.pps.len()))?;
        for it in &self.pps {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\t\t{:x?}", it))?;
        }

        Ok(())
    }
}

impl IO for avcC {
    fn parse(r: &mut BytesMut) -> Self {
        let configuration_version = r.get_u8();
        let profile_indication = r.get_u8();
        let profile_compatibility = r.get_u8();
        let level_indication = r.get_u8();
        let length_size_minus_one = r.get_u8() & 0b11;

        let nb_sps = r.get_u8() & 0b11111;
        let mut sps = Vec::with_capacity(nb_sps as usize);
        for _ in 0..nb_sps {
            let len = r.get_u16();
            sps.push(r.split_to(len as usize));
        }

        let nb_pps = r.get_u8() & 0b11111;
        let mut pps = Vec::with_capacity(nb_sps as usize);
        for _ in 0..nb_pps {
            let len = r.get_u16();
            pps.push(r.split_to(len as usize));
        }

        Self {
            configuration_version,
            profile_indication,
            profile_compatibility,
            level_indication,
            length_size_minus_one,
            sps,
            pps
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u8(self.configuration_version);
        w.put_u8(self.profile_indication);
        w.put_u8(self.profile_compatibility);
        w.put_u8(self.level_indication);
        w.put_u8(self.length_size_minus_one & 0b11);

        w.put_u8((self.sps.len() & 0b11111) as u8);
        for it in &self.sps {
            w.put_u16(it.len() as u16);
            w.put(it.chunk());
        }

        w.put_u8((self.pps.len() & 0b11111) as u8);
        for it in &self.pps {
            w.put_u16(it.len() as u16);
            w.put(it.chunk());
        }

        w
    }
}

#[allow(non_camel_case_types)]
pub struct stts {
    base: FullBox,

    entries: Vec<(u32, u32)>,
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
pub struct stsc {
    base: FullBox,

    entries: Vec<(u32, u32, u32)>,
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
pub struct stsz {
    base: FullBox,

    sample_size: u32,
    entries: Vec<u32>,
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
pub struct stco {
    base: FullBox,

    entries: Vec<u32>,
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

#[allow(non_camel_case_types)]
pub struct moof {
    mfhd: mfhd,
    trafs: Vec<traf>,
}

impl Debug for moof {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t0x6d666864: \"mfhd\""))?;
        f.write_fmt(format_args!("\n{:?}", self.mfhd))?;

        for it in &self.trafs {
            f.write_fmt(format_args!("\n\t0x74726166: \"traf\""))?;
            f.write_fmt(format_args!("\n{:?}", it))?;
        }

        Ok(())
    }
}

impl IO for moof {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            mfhd: mfhd { base: FullBox { version: 0, flags: 0 }, sequence_number: 0 },
            trafs: vec![]
        };

        while 0 < r.len() {
            let mut b = Box::parse(r);

            match b.box_type {
                // mfhd: Movie Fragment Header
                0x6d666864 => {
                    rst.mfhd = mfhd::parse(&mut b.payload);
                }
                // traf: Track Fragment
                0x74726166 => {
                    rst.trafs.push(traf::parse(&mut b.payload));
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Box {
            box_type: 0x6d666864,
            payload: self.mfhd.as_bytes(),
        }.as_bytes());

        for it in self.trafs.iter_mut() {
            w.put(Box {
                box_type: 0x74726166,
                payload: it.as_bytes(),
            }.as_bytes());
        }

        w
    }
}

#[allow(non_camel_case_types)]
pub struct mfhd {
    base: FullBox,

    sequence_number: u32,
}

impl Debug for mfhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\tsequence_number: {:?}", self.sequence_number))?;

        Ok(())
    }
}

impl IO for mfhd {
    fn parse(r: &mut BytesMut) -> Self {
        Self {
            base: FullBox::parse(r),
            sequence_number: r.get_u32(),
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(self.sequence_number);

        w
    }
}

#[allow(non_camel_case_types)]
pub struct traf {
    tfhd: tfhd,
    truns: Vec<trun>,
}

impl Debug for traf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t0x74666864: \"tfhd\""))?;
        f.write_fmt(format_args!("\n{:?}", self.tfhd))?;

        for it in &self.truns {
            f.write_fmt(format_args!("\n\t\t0x7472756e: \"trun\""))?;
            f.write_fmt(format_args!("\n{:?}", it))?;
        }

        Ok(())
    }
}

impl IO for traf {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            tfhd: tfhd {
                base: FullBox { version: 0, flags: 0 },
                track_id: 0,
                base_data_offset: None,
                sample_description_index: None,
                default_sample_duration: None,
                default_sample_size: None,
                default_sample_flags: None
            },
            truns: vec![]
        };

        while 0 < r.len() {
            let mut b = Box::parse(r);

            match b.box_type {
                // tfhd: Track Fragment Header
                0x74666864 => {
                    rst.tfhd = tfhd::parse(&mut b.payload);
                }
                // trun: Track Fragment Run
                0x7472756e => {
                    rst.truns.push(trun::parse(&mut b.payload));
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Box {
            box_type: 0x74666864,
            payload: self.tfhd.as_bytes()
        }.as_bytes());

        for it in self.truns.iter_mut() {
            w.put(Box {
                box_type: 0x7472756e,
                payload: it.as_bytes()
            }.as_bytes());
        }

        w
    }
}

#[allow(non_camel_case_types)]
pub struct tfhd {
    base: FullBox,
    track_id: u32,
    base_data_offset: Option<u64>,
    sample_description_index: Option<u32>,
    default_sample_duration: Option<u32>,
    default_sample_size: Option<u32>,
    default_sample_flags: Option<u32>,
}

impl Debug for tfhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\tflags: {:?}", self.base.flags))?;

        f.write_fmt(format_args!("\n\t\t\ttrack_id: {:?}", self.track_id))?;

        if 0 != (0x000001 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tbase_data_offset: {:?}", self.base_data_offset))?;
        }
        if 0 != (0x000002 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tsample_description_index: {:?}", self.sample_description_index))?;
        }
        if 0 != (0x000008 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_duration: {:?}", self.default_sample_duration))?;
        }
        if 0 != (0x000010 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_size: {:?}", self.default_sample_size))?;
        }
        if 0 != (0x000020 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_flags: {:?}", self.default_sample_flags))?;
        }

        Ok(())
    }
}

impl IO for tfhd {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            track_id: r.get_u32(),
            base_data_offset: None,
            sample_description_index: None,
            default_sample_duration: None,
            default_sample_size: None,
            default_sample_flags: None
        };

        if 0 != (0x000001 & rst.base.flags) {
            rst.base_data_offset = Some(r.get_u64());
        }
        if 0 != (0x000002 & rst.base.flags) {
            rst.sample_description_index = Some(r.get_u32());
        }
        if 0 != (0x000008 & rst.base.flags) {
            rst.default_sample_duration = Some(r.get_u32());
        }
        if 0 != (0x000010 & rst.base.flags) {
            rst.default_sample_size = Some(r.get_u32());
        }
        if 0 != (0x000020 & rst.base.flags) {
            rst.default_sample_flags = Some(r.get_u32());
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        self.base.flags = 0;
        if let Some(_) = self.base_data_offset {
            self.base.flags |= 0x000001;
        }
        if let Some(_) = self.sample_description_index {
            self.base.flags |= 0x000002;
        }
        if let Some(_) = self.default_sample_duration {
            self.base.flags |= 0x000008;
        }
        if let Some(_) = self.default_sample_size {
            self.base.flags |= 0x000010;
        }
        if let Some(_) = self.default_sample_flags {
            self.base.flags |= 0x000020;
        }

        w.put(self.base.as_bytes());

        w.put_u32(self.track_id);

        if let Some(v) = self.base_data_offset {
            w.put_u64(v);
        }
        if let Some(v) = self.sample_description_index {
            w.put_u32(v);
        }
        if let Some(v) = self.default_sample_duration {
            w.put_u32(v);
        }
        if let Some(v) = self.default_sample_size {
            w.put_u32(v);
        }
        if let Some(v) = self.default_sample_flags {
            w.put_u32(v);
        }

        w
    }
}

#[allow(non_camel_case_types)]
pub struct trun {
    base: FullBox,
    data_offset: Option<u32>,
    first_sample_flags: Option<u32>,
    samples: Vec<(Option<u32>, Option<u32>, Option<u32>, Option<u32>)>
}

impl Debug for trun {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\tflags: 0x{:08x?}", self.base.flags))?;

       f.write_fmt(format_args!("\n\t\t\tsample_count: {:?}", self.samples.len()))?;
        if 0 != (0x000001 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tdata_offset: {:?}", self.data_offset))?;
        }
        if 0 != (0x000004 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tfirst_sample_flags: {:?}", self.first_sample_flags))?;
        }

        f.write_fmt(format_args!("\n\t\t\t["))?;
        for (
            sample_duration,
            sample_size,
            sample_flags,
            sample_composition_time_offset,
        ) in &self.samples {
            f.write_fmt(format_args!("\n\t\t\t\t{{"))?;
            if let Some(n) = sample_duration {
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_duration: {:?}", n))?;
            }
            if let Some(n) = sample_size {
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_size: {:?}", n))?;
            }
            if let Some(n) = sample_flags {
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_flags: {:?}", n))?;
            }
            if let Some(n) = sample_composition_time_offset {
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_composition_time_offset: {:?}", n))?;
            }
            f.write_fmt(format_args!("\n\t\t\t\t}}"))?;
        }
        f.write_fmt(format_args!("\n\t\t\t]"))?;

        Ok(())
    }
}

impl IO for trun {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            data_offset: None,
            first_sample_flags: None,
            samples: vec![]
        };

        let sample_count = r.get_u32();

        if 0 != (0x000001 & rst.base.flags) {
            rst.data_offset = Some(r.get_u32());
        }
        if 0 != (0x000004 & rst.base.flags) {
            rst.first_sample_flags = Some(r.get_u32());
        }

        for _ in 0..sample_count {
            let sample_duration = if 0 != (0x000100 & rst.base.flags) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_size = if 0 != (0x000200 & rst.base.flags) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_flags = if 0 != (0x000400 & rst.base.flags) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_composition_time_offset = if 0 != (0x000800 & rst.base.flags) {
                Some(r.get_u32())
            } else {
                None
            };

            rst.samples.push((sample_duration, sample_size, sample_flags, sample_composition_time_offset));
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        self.base.flags = 0;
        if let Some(_) = self.data_offset {
            self.base.flags |= 0x000001;
        }
        if let Some(_) = self.first_sample_flags {
            self.base.flags |= 0x000004;
        }
        if let Some((
                        sample_duration,
                        sample_size,
                        sample_flags,
                        sample_composition_time_offset,
                    )) = self.samples.first() {
            if let Some(_) = *sample_duration {
                self.base.flags |= 0x000100;
            }
            if let Some(_) = *sample_size {
                self.base.flags |= 0x000200;
            }
            if let Some(_) = *sample_flags {
                self.base.flags |= 0x000400;
            }
            if let Some(_) = *sample_composition_time_offset {
                self.base.flags |= 0x000800;
            }
        }

        w.put(self.base.as_bytes());

        w.put_u32(self.samples.len() as u32);

        if let Some(v) = self.data_offset {
            w.put_u32(v);
        }

        if let Some(v) = self.first_sample_flags {
            w.put_u32(v);
        }

        for (
            sample_duration,
            sample_size,
            sample_flags,
            sample_composition_time_offset,
        ) in &self.samples {
            if let Some(n) = sample_duration {
                w.put_u32(*n);
            }
            if let Some(n) = sample_size {
                w.put_u32(*n);
            }
            if let Some(n) = sample_flags {
                w.put_u32(*n);
            }
            if let Some(n) = sample_composition_time_offset {
                w.put_u32(*n);
            }
        }

        w
    }
}
