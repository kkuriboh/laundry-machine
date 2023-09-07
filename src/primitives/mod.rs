use bytemuck::bytes_of;
use sha3::{Digest, Keccak256};

use self::{blob::Blob, block::Block};

pub mod blob;
pub mod block;

pub struct Chain(Vec<Block>, Keccak256);

impl Chain {
    pub fn new(block: Block) -> Self {
        Self(vec![block], Keccak256::new())
    }

    #[inline]
    pub fn blocks(&self) -> &[Block] {
        &self.0
    }

    // TODO: CREATE A HASHER POOL TO REMOVE THIS UNSAFE
    // XXX: SAFETY: be sure to use this behind a lock (UB)
    pub unsafe fn replace(&mut self, chain: Vec<Block>) {
        let mut new_chain = Self(chain, self.1.clone());
        if unsafe { new_chain.validate_chain() } && new_chain.0.len() > self.0.len() {
            *self = new_chain;
            tracing::info!("BLOCKCHAIN WAS REPLACED");
            return;
        }
        tracing::warn!("RECEIVED INVALID OR INCOMPLETE CHAIN");
    }

    pub fn generate_next_block(&mut self, data: Blob) -> Block {
        let block = self.0.last().unwrap();
        let new_block = Block::new(block.index + 1, data, block.hash, &mut self.1);
        self.0.push(new_block);

        tracing::info!("BLOCK CREATED {}", new_block.index);

        new_block
    }

    pub fn push_block(&mut self, block: Block) -> bool {
        let previous_block = self.0.last().unwrap();
        let is_valid = block.validate(previous_block, &mut self.1);
        if is_valid {
            self.0.push(block);
        }

        is_valid
    }

    // TODO: CREATE A HASHER POOL TO REMOVE THIS UNSAFE
    // XXX: SAFETY: be sure to use this behind a lock (UB)
    pub unsafe fn validate_chain(&mut self) -> bool {
        let hasher = &mut self.1 as *mut _;
        let blocks = self.blocks();

        if bytes_of(&blocks[0]) != bytes_of(&Block::GENESIS) {
            return false;
        }

        !blocks
            .array_windows::<2>()
            .any(|&[previous, next]| !next.validate(&previous, unsafe { &mut *hasher }))
    }
}
