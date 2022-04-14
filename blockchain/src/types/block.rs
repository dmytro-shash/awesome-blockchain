use chrono::Utc;

use crate::types::Hash;
use crate::types::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: Hash,
    pub previous_hash: Option<Hash>,
    pub transactions: Vec<Transaction>,
    pub timestamp: i64,
    pub nonce: u64,
}