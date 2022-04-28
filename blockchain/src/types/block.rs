use crate::types::transaction::Transaction;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type BlockHash = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub nonce: u64,
    pub previous_hash: Option<BlockHash>,
    pub hash: BlockHash,
    pub transactions: Vec<Transaction>,
}

impl Block {
    // Create a new block. The hash value will be calculated and set automatically.
    pub fn new(
        index: u64,
        nonce: u64,
        previous_hash: Option<BlockHash>,
        transactions: Vec<Transaction>,
    ) -> Block {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp_millis(),
            nonce,
            previous_hash,
            hash: BlockHash::default(),
            transactions,
        };
        block.hash = block.calculate_hash();

        block
    }

    pub fn calculate_hash(&self) -> BlockHash {
        let mut block_data = self.clone();
        block_data.hash = String::default();
        let serialized_block_data = serde_json::to_string(&block_data).unwrap();
        // Calculate and return SHA-256 hash value.
        let mut hasher = Sha256::new();
        hasher.update(serialized_block_data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_transaction(sender: String, recipient: String, amount: u64) -> Transaction {
        Transaction {
            sender,
            recipient,
            amount,
        }
    }

    #[test]
    fn test_create_block_with_transactions() {
        let transaction_1 =
            create_mock_transaction("alice.near".to_owned(), "bob.near".to_owned(), 10);
        let transaction_2 =
            create_mock_transaction("bob.near".to_owned(), "alice.near".to_owned(), 5);

        let block = Block::new(0, 10, None, vec![transaction_1, transaction_2]);

        assert_eq!(block.previous_hash, None);
        assert!(!block.transactions.is_empty());
    }
}
