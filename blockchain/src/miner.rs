use crate::types::block::Block;
use crate::types::transaction_pool::TransactionVec;
use crate::{Blockchain, Context, TransactionPool};
use log::{error, info};

use crate::util::execution::{sleep_millis, Runnable};
use anyhow::Result;
use thiserror::Error;

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
        info!(
            "start mining with difficulty {}",
            self.blockchain.difficulty
        );

        // In each loop it tries to find the next valid block and append it to the blockchain
        let mut block_counter = 0;
        loop {
            if self.must_stop_mining(block_counter) {
                info!("block limit reached, stopping mining");
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
                    self.blockchain.add_block(block.clone()).unwrap();
                    block_counter += 1;
                }
                None => {
                    let index = last_block.index + 1;
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
            if next_block
                .hash
                .starts_with(&"0".repeat(self.target as usize))
            {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::block::BlockHash;
    use crate::types::transaction::Transaction;

    // We use SHA 256 hashes
    const MAX_DIFFICULTY: u32 = 256;

    #[test]
    fn test_create_next_block() {
        let miner = create_default_miner();
        let block = create_empty_block();

        let next_block = miner.create_next_block(&block, Vec::new(), 0);

        // the next block must follow the previous one
        assert_eq!(next_block.index, block.index + 1);
        assert_eq!(next_block.previous_hash.unwrap(), block.hash);
    }

    #[test]
    fn test_mine_block_found() {
        // let's use a small difficulty target for fast testing
        let difficulty = 1;

        // this should be more than enough nonce's to find a block with only 1 zero
        let max_nonce = 1_000;

        // check that the block is mined
        let miner = create_miner(difficulty, max_nonce);
        let last_block = create_empty_block();
        let result = miner.mine_block(&last_block, Vec::new());
        assert!(result.is_some());

        // check that the block is valid
        let mined_block = result.unwrap();
        assert_mined_block_is_valid(&mined_block, &last_block, difficulty);
    }

    #[test]
    fn test_mine_block_not_found() {
        // let's use a high difficulty target to never find a block
        let difficulty = MAX_DIFFICULTY;

        // with a max_nonce so low, we will never find a block
        // and also the test will end fast
        let max_nonce = 10;

        // check that the block is not mined
        let miner = create_miner(difficulty, max_nonce);
        let last_block = create_empty_block();
        let result = miner.mine_block(&last_block, Vec::new());
        assert!(result.is_none());
    }

    #[test]
    fn test_run_block_found() {
        // with a max_nonce so high and difficulty so low
        // we will always find a valid block
        let difficulty = 1;
        let max_nonce = 1_000_000;
        let miner = create_miner(difficulty, max_nonce);

        let blockchain = miner.blockchain.clone();
        let transaction_pool = miner.transaction_pool.clone();

        add_mock_transaction(&transaction_pool);
        let result = miner.run();

        // mining should be successful
        assert!(result.is_ok());

        // a new block should have been added to the blockchain
        let blocks = blockchain.get_all_blocks();
        assert_eq!(blocks.len(), 2);
        let genesis_block = &blocks[0];
        let mined_block = &blocks[1];

        // the mined block must be valid
        assert_mined_block_is_valid(mined_block, genesis_block, blockchain.difficulty);

        // the mined block must include the transaction added previously
        let mined_transactions = &mined_block.transactions;
        assert_eq!(mined_transactions.len(), 1);

        // the transaction pool must be empty
        // because the transaction was added to the block when mining
        let transactions = transaction_pool.pop();
        assert!(transactions.is_empty());
    }

    #[test]
    #[should_panic(expected = "No valid block was mined at index `1`")]
    fn test_run_block_not_found() {
        // with a max_nonce so low and difficulty so high
        // we will never find a valid block
        let difficulty = MAX_DIFFICULTY;
        let max_nonce = 1;
        let miner = create_miner(difficulty, max_nonce);

        let transaction_pool = &miner.transaction_pool;
        add_mock_transaction(transaction_pool);

        // mining should return a BlockNotMined error
        miner.run().unwrap();
    }

    fn create_default_miner() -> Miner {
        let difficulty = 1;
        let max_nonce = 1;
        create_miner(difficulty, max_nonce)
    }

    fn create_miner(difficulty: u32, max_nonce: u64) -> Miner {
        let max_blocks = 1;
        let tx_waiting_ms = 1;

        let blockchain = Blockchain::new(difficulty);
        let transaction_pool = TransactionPool::new();

        Miner {
            max_blocks,
            max_nonce,
            tx_waiting_ms,
            blockchain,
            transaction_pool,
            target: difficulty,
        }
    }

    fn create_empty_block() -> Block {
        return Block::new(0, 0, Some(BlockHash::default()), Vec::new());
    }

    fn add_mock_transaction(pool: &TransactionPool) {
        let transaction = Transaction {
            sender: "1".to_string(),
            recipient: "2".to_string(),
            amount: 3,
        };
        pool.add_transaction(transaction.clone());
    }

    fn assert_mined_block_is_valid(mined_block: &Block, previous_block: &Block, difficulty: u32) {
        assert_eq!(mined_block.index, previous_block.index + 1);
        assert_eq!(
            mined_block.previous_hash.as_ref().unwrap(),
            &previous_block.hash
        );
        assert!(mined_block
            .hash
            .starts_with(&"0".repeat(difficulty as usize)));
    }
}
