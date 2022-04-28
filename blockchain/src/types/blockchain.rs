use crate::types::block::Block;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::util::response::Response;

pub type BlockVec = Vec<Block>;

type SyncedBlockVec = Arc<Mutex<BlockVec>>;

// Struct that holds all the blocks in the blockchain
// Multiple threads can read/write concurrently to the list of blocks
#[derive(Debug, Clone)]
pub struct Blockchain {
    pub difficulty: u32,
    blocks: SyncedBlockVec,
}

impl Blockchain {
    // Creates a new blockchain with a genesis block
    pub fn new(difficulty: u32) -> Blockchain {
        let genesis_block = Blockchain::create_genesis_block();

        // add the genesis block to the synced vec of blocks
        let blocks = vec![genesis_block];

        let synced_blocks = Arc::new(Mutex::new(blocks));

        Blockchain {
            difficulty,
            blocks: synced_blocks,
        }
    }

    // Returns a copy of the most recent block in the blockchain
    pub fn get_last_block(&self) -> Block {
        let blocks = self.blocks.lock().unwrap();

        blocks[blocks.len() - 1].clone()
    }

    // Returns a copy of the whole list of blocks
    pub fn get_all_blocks(&self) -> BlockVec {
        let blocks = self.blocks.lock().unwrap();

        blocks.clone()
    }

    // Returns a block by index
    pub(crate) fn get_block_by_index(&self, index: u64) -> Response {
        let blocks = self.get_all_blocks();
        let mut block_hash_map = HashMap::new();

        for (internal_index, block) in blocks.iter().enumerate() {
            block_hash_map.insert(internal_index as u64, block.clone());
        }

        match block_hash_map.get(&index) {
            None => Response::new(false, "there is no such a block".to_string()),
            Some(block) => Response::new(true, format!("{:?}", block))
        }
    }

    // adding new block into blockchain
    pub fn add_block(&self, block: Block) -> Result<(), &str> {
        let mut blocks = self.blocks.lock().unwrap();
        let last = &blocks[blocks.len() - 1];

        // check that the index is valid
        if block.index != last.index + 1 {
            return Err("invalid index");
        }

        // check that the previous_hash is valid
        if block.previous_hash.as_ref().unwrap().clone() != last.hash {
            return Err("invalid previous hash");
        }

        // check that the hash matches the data
        if block.hash != block.calculate_hash() {
            return Err("invalid hash");
        }

        // check that the target is correct
        if !block
            .hash
            .starts_with(&"0".repeat(self.difficulty as usize))
        {
            return Err("invalid target");
        }

        // append the block to the end
        blocks.push(block);

        Ok(())
    }

    fn create_genesis_block() -> Block {
        let mut block = Block::new(0, 0, None, vec![]);

        block.timestamp = 0;
        block.hash = block.calculate_hash();

        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NO_TARGET: u32 = 0;

    #[test]
    fn is_valid_genesis_block() {
        let blockchain = Blockchain::new(NO_TARGET);

        let blocks = blockchain.get_all_blocks();
        assert_eq!(blocks.len(), 1);

        let block = blockchain.get_last_block();
        assert_eq!(block.hash, blocks[0].hash);

        assert_eq!(block.index, 0);
        assert_eq!(block.nonce, 0);
        assert_eq!(block.previous_hash, None);
        assert!(block.transactions.is_empty());
    }
}
