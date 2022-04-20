mod types;
mod util;
mod miner;

use util::context::Context;
use util::config::Config;
use crate::miner::Miner;
use crate::types::blockchain::Blockchain;
use crate::types::transaction_pool::TransactionPool;
use crate::util::execution;

fn main() {
    // reading config from config.json
    let config = Config::read_config_from_file("config.json").unwrap();

    let difficulty = config.difficulty;
    let context = Context {
        config,
        blockchain: Blockchain::new(difficulty),
        pool: TransactionPool::new(),
    };

    // initialize the processes
    let miner = Miner::new(&context);

    execution::run_in_parallel(vec![miner]);
}
