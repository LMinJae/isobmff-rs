use bytes::BytesMut;

pub trait IO {
    fn parse(r: &mut BytesMut) -> Self;
    fn as_bytes(&mut self) -> BytesMut;
}
