use crate::types::transaction::Transaction;
use log::info;
use std::sync::{Arc, Mutex};

pub type TransactionVec = Vec<Transaction>;

type SyncedTransactionVec = Arc<Mutex<TransactionVec>>;

#[derive(Debug, Clone)]
pub struct TransactionPool {
    transactions: SyncedTransactionVec,
}

impl TransactionPool {
    // Creates a empty transaction pool
    pub fn new() -> TransactionPool {
        TransactionPool {
            transactions: SyncedTransactionVec::default(),
        }
    }

    // Adds a new transaction to the pool
    pub fn add_transaction(&self, transaction: Transaction) {
        let mut transactions = self.transactions.lock().unwrap();
        transactions.push(transaction);
        info!("transaction added");
    }

    // Returns a copy of all transactions and empties the pool
    // This operation is safe to be called concurrently from multiple threads
    // because its protected by Arc<Mutex>
    pub fn pop(&self) -> TransactionVec {
        let mut transactions = self.transactions.lock().unwrap();
        let transactions_clone = transactions.clone();
        transactions.clear();

        transactions_clone
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_transaction(amount: u64) -> Transaction {
        Transaction {
            sender: "alice.near".to_string(),
            recipient: "bob.near".to_string(),
            amount,
        }
    }

    #[test]
    fn transaction_pool_should_be_empty() {
        let transaction_pool = TransactionPool::new();

        let transactions = transaction_pool.pop();
        assert!(transactions.is_empty());
    }

    #[test]
    fn transaction_pool_contains_one_transaction() {
        let transaction_pool = TransactionPool::new();

        // add a new transaction to the pool
        let transaction = create_mock_transaction(1);
        transaction_pool.add_transaction(transaction.clone());

        // pop the values and check that the transaction is included
        let mut transactions = transaction_pool.pop();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].amount, transaction.amount);

        // after the previous pop, the pool should still be empty
        transactions = transaction_pool.pop();
        assert!(transactions.is_empty());
    }

    #[test]
    fn transaction_pool_contains_several_transaction() {
        let transaction_pool = TransactionPool::new();

        // add a new transaction to the pool
        let transaction_a = create_mock_transaction(10);
        let transaction_b = create_mock_transaction(12);
        transaction_pool.add_transaction(transaction_a.clone());
        transaction_pool.add_transaction(transaction_b.clone());

        // pop the values and check that the transactions are included
        let mut transactions = transaction_pool.pop();
        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].amount, transaction_a.amount);
        assert_eq!(transactions[1].amount, transaction_b.amount);

        // after the previous pop, the pool should still be empty
        transactions = transaction_pool.pop();
        assert!(transactions.is_empty());
    }
}
