use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Bytes<'a> {
    Slice(&'a [u8]),
    Bytes(bytes::Bytes),
    Vec(Rc<Vec<u8>>),
    String(Rc<String>),
}


