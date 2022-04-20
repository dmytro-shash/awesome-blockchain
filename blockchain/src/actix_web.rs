use crate::execution::Runnable;
use crate::types::block::Block;
use crate::types::transaction::Transaction;
use crate::{Blockchain, Context, TransactionPool};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use log::info;

struct ServerData {
    blockchain: Blockchain,
    pool: TransactionPool,
}

pub struct Server {
    port: u16,
    blockchain: Blockchain,
    pool: TransactionPool,
}

impl Runnable for Server {
    fn run(&self) -> Result<()> {
        let api_blockchain = self.blockchain.clone();
        let api_pool = self.pool.clone();

        start_blockchain_server(self.port, api_blockchain, api_pool)
    }
}

impl Server {
    pub fn new(context: &Context) -> Server {
        Server {
            port: context.config.port,
            blockchain: context.blockchain.clone(),
            pool: context.pool.clone(),
        }
    }
}

#[actix_web::main]
async fn start_blockchain_server(
    port: u16,
    blockchain: Blockchain,
    pool: TransactionPool,
) -> Result<()> {
    let url = format!("localhost:{}", port);
    // These variables are really "Arc" pointers to a shared memory value
    // So when we clone them, we are only cloning the pointers and not the actual data
    let server_data = web::Data::new(ServerData { blockchain, pool });

    HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .route("/blocks/all", web::get().to(get_blocks))
            .route("/block/add", web::post().to(add_block))
            .route("/transactions/new/{from}/{to}/{amount}", web::post().to(add_transaction))
    })
        .bind(url)
        .unwrap()
        .run()
        .await?;

    Ok(())
}

async fn get_blocks(state: web::Data<ServerData>) -> impl Responder {
    let blockchain = &state.blockchain;
    let blocks = blockchain.get_all_blocks();

    HttpResponse::Ok().json(&blocks)
}

// Adds a new block to the blockchain
async fn add_block(state: web::Data<ServerData>, block_json: web::Json<Block>) -> HttpResponse {
    let mut block = block_json.into_inner();

    // The hash of the block is mandatory and the blockchain checks if it's correct
    // That's a bit inconvenient for manual use of the API
    // So we ignore the coming hash and recalculate it again before adding to the blockchain
    block.hash = block.calculate_hash();

    let blockchain = &state.blockchain;
    let result = blockchain.add_block(block.clone());

    match result {
        Ok(_) => {
            info!("Received new block {}", block.index);
            HttpResponse::Ok().finish()
        }
        Err(error) => HttpResponse::BadRequest().body(error.to_string()),
    }
}

// Adds a new transaction to the pool, to be included on the next block
async fn add_transaction(
    state: web::Data<ServerData>,
    transaction: web::Path<(String, String, u64)>,
) -> impl Responder {
    let (sender, recipient, amount) = transaction.into_inner();

    let transaction = Transaction {
        sender,
        recipient,
        amount,
    };

    let pool = &state.pool;
    pool.add_transaction(transaction.clone());

    HttpResponse::Ok().json(&transaction)
}
