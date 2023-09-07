use std::{mem::size_of, time::SystemTime};

use bytemuck::{bytes_of, Pod, Zeroable};
use sha3::{digest::FixedOutputReset, Digest, Keccak256};

use super::blob::Blob;

pub type Hash = [u8; 32];

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Block {
    pub index: usize,
    pub data: Blob,
    pub timestamp: u64,
    pub hash: Hash,
    pub previous_hash: Hash,
}

#[inline]
fn timestamp() -> u64 {
    SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs()
}

impl Block {
    pub const GENESIS: Block = Block {
        index: 0,
        data: Blob::new(),
        timestamp: 1692981149,
        previous_hash: [0; 32],
        hash: [
            33, 252, 152, 207, 112, 38, 90, 233, 63, 235, 108, 237, 45, 43, 3, 25, 33, 70, 114,
            108, 82, 24, 61, 146, 143, 127, 139, 120, 215, 118, 75, 177,
        ],
    };

    pub fn new(index: usize, data: Blob, previous_hash: Hash, hasher: &mut Keccak256) -> Self {
        let timestamp = timestamp();
        Self {
            index,
            data,
            timestamp,
            hash: Self::calc_hash(index, &previous_hash, timestamp, &data, hasher),
            previous_hash,
        }
    }

    pub fn validate(&self, previous_block: &Block, hasher: &mut Keccak256) -> bool {
        self.index == previous_block.index + 1
            && self.previous_hash == previous_block.hash
            && self.hash_from_block(&previous_block.hash, hasher) == self.hash
    }

    // TODO: unit test this
    fn hashable_bytes(
        index: usize,
        previous_hash: &Hash,
        timestamp: u64,
        data: &Blob,
    ) -> [u8; size_of::<usize>() + size_of::<Hash>() + size_of::<u64>() + size_of::<Blob>()] {
        const SIZE_OF_U64: usize = size_of::<u64>();
        const SIZE_OF_USIZE: usize = size_of::<usize>();
        const SIZE_OF_HASH: usize = size_of::<Hash>();
        const SIZE_OF_BLOB: usize = size_of::<Blob>();

        let mut buff = [0; SIZE_OF_U64 + SIZE_OF_USIZE + SIZE_OF_HASH + SIZE_OF_BLOB];

        let idx = index.to_ne_bytes();
        let timestamp = timestamp.to_ne_bytes();
        let blob = bytes_of(data);

        buff[..SIZE_OF_USIZE].copy_from_slice(&idx);
        buff[SIZE_OF_USIZE..SIZE_OF_USIZE + SIZE_OF_U64].copy_from_slice(&timestamp);
        buff[SIZE_OF_USIZE + SIZE_OF_U64..SIZE_OF_USIZE + SIZE_OF_U64 + SIZE_OF_BLOB]
            .copy_from_slice(blob);
        buff[SIZE_OF_USIZE + SIZE_OF_U64 + SIZE_OF_BLOB..].copy_from_slice(previous_hash);

        buff
    }

    fn calc_hash(
        index: usize,
        previous_hash: &Hash,
        timestamp: u64,
        data: &Blob,
        hasher: &mut Keccak256,
    ) -> Hash {
        hasher.update(Self::hashable_bytes(index, previous_hash, timestamp, data));
        hasher.finalize_fixed_reset().into()
    }

    fn hash_from_block(&self, previous_hash: &Hash, hasher: &mut Keccak256) -> Hash {
        hasher.update(Self::hashable_bytes(
            self.index,
            previous_hash,
            self.timestamp,
            &self.data,
        ));
        hasher.finalize_fixed_reset().into()
    }
}
