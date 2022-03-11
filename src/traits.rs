use bytes::BytesMut;

pub trait IO {
    fn len(&self) -> usize;

    fn parse(r: &mut BytesMut) -> Self;
    fn as_bytes(&mut self) -> BytesMut;
}
