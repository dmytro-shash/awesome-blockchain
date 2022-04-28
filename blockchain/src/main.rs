mod actix_web;
mod miner;
mod types;
mod util;

use crate::actix_web::Server;
use crate::execution::set_ctrlc_handler;
use crate::miner::Miner;
use crate::types::blockchain::Blockchain;
use crate::types::transaction_pool::TransactionPool;
use crate::util::execution;
use util::config::Config;
use util::context::Context;

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
    let actix_server = Server::new(&context);

    set_ctrlc_handler();

    execution::run_in_parallel(vec![Box::new(miner), Box::new(actix_server)]);
}
