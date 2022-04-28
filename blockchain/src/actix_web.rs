use crate::execution::Runnable;
use crate::types::block::Block;
use crate::types::transaction::Transaction;
use crate::{Blockchain, Context, TransactionPool};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use anyhow::Result;

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
        let server_blockchain = self.blockchain.clone();
        let server_transaction_pool = self.pool.clone();

        start_blockchain_server(self.port, server_blockchain, server_transaction_pool)
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
            .route("/blocks", web::get().to(get_blocks))
            .route("/blocks", web::post().to(add_block))
            .route("/blocks/get/{index}", web::get().to(get_block_by_index))
            .route("/tx/pool", web::get().to(get_transactions))
            .route(
                "/tx/new/{from}/{to}/{amount}",
                web::get().to(add_transaction),
            )
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

async fn get_block_by_index(state: web::Data<ServerData>, index: web::Path<u64>) -> impl Responder {
    let blockchain = &state.blockchain;

    HttpResponse::Ok().json(&blockchain.get_block_by_index(index.into_inner()))
}

async fn get_transactions(state: web::Data<ServerData>) -> impl Responder {
    let transactions = &state.pool.pop();
    HttpResponse::Ok().json(&transactions)
}

// Adds a new block to the blockchain
async fn add_block(state: web::Data<ServerData>, block_json: web::Json<Block>) -> impl Responder {
    let mut block = block_json.into_inner();

    block.hash = block.calculate_hash();

    let blockchain = &state.blockchain;
    let result = blockchain.add_block(block.clone());

    match result {
        Ok(_) => {
            HttpResponse::Ok().finish()
        }
        Err(error) => HttpResponse::BadRequest().body(error.to_string()),
    }
}

async fn add_transaction(
    state: web::Data<ServerData>,
    transaction: web::Path<(String, String, u64)>,
) -> String {
    let (sender, recipient, amount) = transaction.into_inner();

    let transaction = Transaction {
        sender,
        recipient,
        amount,
    };
    let pool = &state.pool;
    pool.add_transaction(transaction.clone());

    format!("new transaction {:?}!", transaction)
}
