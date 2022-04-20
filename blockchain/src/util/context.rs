use crate::Config;

use crate::types::blockchain::Blockchain;
use crate::types::transaction_pool::TransactionPool;

pub struct Context {
    pub config: Config,
    pub blockchain: Blockchain,
    pub pool: TransactionPool,
}
