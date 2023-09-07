use std::fmt::Display;

use bytemuck::{Pod, Zeroable};

use crate::generic_helpers::{Assert, IsTrue};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Blob {
    data: [u8; 1024],
    cursor: usize,
}

const ZEROED_BLOB: Blob = Blob {
    data: [0; 1024],
    cursor: 0,
};

impl Blob {
    pub const fn new() -> Self {
        ZEROED_BLOB
    }

    pub fn _write(&mut self, buf: &[u8]) -> usize {
        let len = buf.len();
        assert!(len <= 1024 - self.cursor);

        for (idx, b) in buf.iter().enumerate() {
            self.data[self.cursor + idx] = *b;
        }

        self.cursor += len;

        len
    }

    pub fn _zero(&mut self) {
        *self = ZEROED_BLOB;
    }

    pub(crate) fn buffer(&self) -> &[u8] {
        &self.data
    }
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl From<([u8; 1024], usize)> for Blob {
    fn from((data, cursor): ([u8; 1024], usize)) -> Self {
        Self { data, cursor }
    }
}

impl<const T: usize> From<&[u8; T]> for Blob
where
    Assert<{ T <= 1024 }>: IsTrue,
{
    fn from(value: &[u8; T]) -> Self {
        let mut ret = Blob::new();

        for (idx, b) in value.iter().enumerate() {
            ret.data[idx] = *b;
        }

        ret.cursor = T;
        ret
    }
}

impl<const T: usize> From<[u8; T]> for Blob
where
    Assert<{ T <= 1024 }>: IsTrue,
{
    fn from(value: [u8; T]) -> Self {
        let mut ret = Blob::new();

        for (idx, b) in value.into_iter().enumerate() {
            ret.data[idx] = b;
        }

        ret.cursor = T;
        ret
    }
}

impl From<&[u8]> for Blob {
    fn from(value: &[u8]) -> Self {
        let len = value.len();
        assert!(len <= 1024);

        let mut ret = Blob::new();

        for (idx, b) in value.iter().enumerate() {
            ret.data[idx] = *b;
        }

        ret.cursor = len;
        ret
    }
}
