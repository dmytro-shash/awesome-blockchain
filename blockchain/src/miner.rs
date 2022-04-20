use log::{error, info};
use crate::{Blockchain, Context, TransactionPool};
use crate::types::block::{Block, BlockHash};
use crate::types::transaction_pool::TransactionVec;

use anyhow::Result;
use thiserror::Error;
use crate::util::execution::{Runnable, sleep_millis};

#[derive(Error, Debug)]
pub enum MinerError {
    #[error("No valid block was mined at index `{0}`")]
    BlockNotMined(u64),
}


pub struct Miner {
    max_blocks: u64,
    max_nonce: u64,
    tx_waiting_ms: u64,
    blockchain: Blockchain,
    transaction_pool: TransactionPool,
    target: u32,
}

impl Runnable for Miner {
    fn run(&self) -> Result<()> {
        self.start()
    }
}


impl Miner {
    pub fn new(context: &Context) -> Miner {
        Miner {
            max_blocks: context.config.max_blocks,
            max_nonce: context.config.max_nonce,
            tx_waiting_ms: context.config.tx_waiting_ms,
            blockchain: context.blockchain.clone(),
            transaction_pool: context.pool.clone(),
            target: context.config.difficulty,
        }
    }


    // Try to constantly calculate and append new valid blocks to the blockchain,
    // including all pending transactions in the transaction pool each time
    pub fn start(&self) -> Result<()> {
        println!(
            "start mining with difficulty {}",
            self.blockchain.difficulty
        );

        // In each loop it tries to find the next valid block and append it to the blockchain
        let mut block_counter = 0;
        loop {
            if self.must_stop_mining(block_counter) {
                println!("block limit reached, stopping mining");
                return Ok(());
            }

            // Empty all transactions from the pool, they will be included in the new block
            let transactions = self.transaction_pool.pop();

            // Do not try to mine a block if there are no transactions in the pool
            if transactions.is_empty() {
                sleep_millis(self.tx_waiting_ms);
                continue;
            }

            // try to find a valid next block of the blockchain
            let last_block = self.blockchain.get_last_block();
            let mining_result = self.mine_block(&last_block, transactions.clone());
            match mining_result {
                Some(block) => {
                    println!("valid block found for index {}", block.index);
                    self.blockchain.add_block(block.clone())?;
                    block_counter += 1;
                }
                None => {
                    let index = last_block.index + 1;
                    error!("no valid block was found for index {}", index);
                    return Err(MinerError::BlockNotMined(index).into());
                }
            }
        }
    }

    // check if we have hit the limit of mined blocks (if the limit is set)
    fn must_stop_mining(&self, block_counter: u64) -> bool {
        self.max_blocks > 0 && block_counter >= self.max_blocks
    }


    // Tries to find the next valid block of the blockchain
    // It will create blocks with different "nonce" values until one has a hash that matches the difficulty
    // Returns either a valid block (that satisfies the difficulty) or "None" if no block was found
    fn mine_block(&self, last_block: &Block, transactions: TransactionVec) -> Option<Block> {
        for nonce in 0..self.max_nonce {
            let next_block = self.create_next_block(last_block, transactions.clone(), nonce);

            // A valid block must have a hash with enough starting zeroes with represents as target
            if next_block.hash.starts_with(&"0".repeat(self.target as usize)) {
                return Some(next_block);
            }
        }

        None
    }

    // Creates a valid next block for a blockchain
    // Takes into account the index and the hash of the previous block
    fn create_next_block(
        &self,
        last_block: &Block,
        transactions: TransactionVec,
        nonce: u64,
    ) -> Block {
        let index = (last_block.index + 1) as u64;
        let previous_hash = last_block.clone().hash;

        // hash of the new block is automatically calculated on creation
        Block::new(index, nonce, Some(previous_hash), transactions)
    }
}