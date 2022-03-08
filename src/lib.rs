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

pub struct FullBox {
    pub version: u8,
    pub flags: u32,
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
    pub major_brand: u32,
    pub minor_version: u32,
    pub compatible_brands: Vec<u32>,
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
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub rate: u32,
    pub volume: u16,
    pub matrix: [u32; 9],
    pub next_track_id: u32,
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

    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub duration: u64,
    pub layer: u16,
    pub alternate_group: u16,
    pub volume: u16,
    matrix: [u32; 9],
    pub width: u32,
    pub height: u32,
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

    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: u16,
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

    pub handler_type: u32,
    pub name: String,
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
pub struct vmhd {
    base: FullBox,

    pub graphicsmode: u16,
    pub opcolor: [u16; 3],
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
pub struct smhd {
    base: FullBox,

    pub balance: u16,
}

impl IO for smhd {
    fn parse(r: &mut BytesMut) -> Self {
        let base = FullBox::parse(r);

        Self {
            base,
            balance: r.get_u16(),
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u16(self.balance);

        w
    }
}

#[allow(non_camel_case_types)]
pub struct dinf {
    pub dref: dref,
}

impl IO for dinf {
    fn parse(r: &mut BytesMut) -> Self {
        let mut b = Box::parse(r);

        let mut rst = Self {
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

#[allow(non_camel_case_types)]
pub struct dref {
    pub base: FullBox,

    // XXX
    pub entries: Vec<url_>,
}

impl IO for dref {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            entries: vec![]
        };

        let entry_count = r.get_u32();

        for _ in 0..entry_count {
            let mut b = Box::parse(r);
            match b.box_type {
                // url : URL
                0x75726c20 => {
                    rst.entries.push(url_::parse(&mut b.payload));
                }
                _ => {
                }
            }
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
pub struct url_ {
    pub base: FullBox,

    location: String,
}

impl IO for url_ {
    fn parse(r: &mut BytesMut) -> Self {
        let base = FullBox::parse(r);

        Self {
            base,
            location: std::str::from_utf8(r.split_to(r.len()).chunk()).unwrap().to_owned(),
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put(self.location.as_bytes());

        w
    }
}

#[allow(non_camel_case_types)]
pub struct stts {
    base: FullBox,

    pub entries: Vec<(u32, u32)>,
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
pub struct stsz {
    base: FullBox,

    pub sample_size: u32,
    pub entries: Vec<u32>,
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
pub struct stsc {
    base: FullBox,

    pub entries: Vec<(u32, u32, u32)>,
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
pub struct stco {
    base: FullBox,

    pub entries: Vec<u32>,
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
pub struct mfhd {
    base: FullBox,

    pub sequence_number: u32,
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
pub struct tfhd {
    pub base: FullBox,
    pub track_id: u32,
    pub base_data_offset: Option<u64>,
    pub sample_description_index: Option<u32>,
    pub default_sample_duration: Option<u32>,
    pub default_sample_size: Option<u32>,
    pub default_sample_flags: Option<u32>,
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

        // optional
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
    pub base: FullBox,
    pub data_offset: Option<u32>,
    pub first_sample_flags: Option<u32>,
    pub samples: Vec<(Option<u32>, Option<u32>, Option<u32>, Option<u32>)>
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

        // optional
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
