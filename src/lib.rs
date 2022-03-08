use bytes::{Buf, BufMut, BytesMut};

pub trait IO {
    fn parse(r: &mut BytesMut) -> Self;
    fn as_bytes(self) -> BytesMut;
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

    fn as_bytes(self) -> BytesMut {
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
        w.put(self.payload);

        w
    }
}

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

    fn as_bytes(self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u32(self.major_brand);
        w.put_u32(self.minor_version);
        for it in self.compatible_brands.iter() {
            w.put_u32(*it);
        }

        w
    }
}

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

    fn as_bytes(self) -> BytesMut {
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
        for it in self.matrix.iter() {
            w.put_u32(*it);
        }
        w.put_u64(0);
        w.put_u64(0);
        w.put_u64(0);
        w.put_u32(self.next_track_id);

        w
    }
}

pub struct tkhd {
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
        let version = r.get_u8();
        let _flags = r.split_to(3);

        let mut rst = Self::default();

        {
            let (
                creation_time,
                modification_time,
                track_id,
                _,
                duration,
            ) = if 1 == version {
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

    fn as_bytes(self) -> BytesMut {
        let mut w = BytesMut::new();

        if (u32::MAX as u64) < self.creation_time ||
            (u32::MAX as u64) < self.modification_time ||
            (u32::MAX as u64) < self.duration {
            w.put_u32(0x01000000);

            w.put_u64(self.creation_time);
            w.put_u64(self.modification_time);
            w.put_u32(self.track_id);
            w.put_u32(0);
            w.put_u64(self.duration);
        } else {
            w.put_u32(0);

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
        for it in self.matrix.iter() {
            w.put_u32(*it);
        }
        w.put_u32(self.width);
        w.put_u32(self.height);

        w
    }
}

pub struct mdhd {
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

    fn as_bytes(self) -> BytesMut {
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

        w.put_u16(self.language);
        w.put_u16(0);

        w
    }
}


pub struct hdlr {
    pub handler_type: u32,
    pub name: String,
}

impl IO for hdlr {
    fn parse(r: &mut BytesMut) -> Self {
        let _ = r.get_u32();

        let _ = r.get_u32();
        let handler_type = r.get_u32();
        let _ = r.split_to(12);
        let name = std::str::from_utf8(r.split_to(r.len() - 1).chunk()).unwrap().to_string();

        Self {
            handler_type,
            name,
        }
    }

    fn as_bytes(self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u32(0);

        w.put_u32(0);
        w.put_u32(self.handler_type);
        w.put_u32(0);
        w.put_u32(0);
        w.put_u32(0);
        w.put(self.name.as_bytes());
        w.put_u8(0);

        w
    }
}
