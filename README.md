# PoW Blockchain Test Assignment 🦒

## General information

Have several structures that represent simplified Blockchain

1. [Transaction](#Transaction)
2. [Transaction Pool](#Transaction Pool)
3. [Block](#Block)
4. [Blockchain](#Blockchain)

### Transaction

Each transaction has a **sender**, **recipient** and **amount**

### Transaction Pool

All unprocessed transactions are located in transaction pool, until miner form them into new block

### Block

Each block contains the following data:

* **index**: position of the block in the blockchain
* **timestamp**: date and time of block creation
* **nonce**: arbitrary number that makes the block, when hashed, meet the mining difficulty restriction. Is the number
  that miners are competing to get first
* **previous_hash**: hash of the previous block in the chain. Allows to maintain order of blocks in the blockchain.
  There is an exception with the first block of the chain (genesis block) which has no previous_hash
* **hash**: hash of the block including all fields
* **transactions**: a list of all transactions included in the block.

### Blockchain

Blockchain contains

* **target**: hash of new blocks has to satisfy the difficulty constraint, which is to be less than a target value
* **blocks**: a list of all blocks included in the blockchain

We encapsulate the proceeding and adding transactions
and blocks using `Arc<Mutex<obj>>` so that we can control the order of adding to transaction pool and blockchain
respectively.

## API interaction

### configuring your app with parameters in config.json:

```
  {
     "port": port your want to run your app ( i chose 8000 )
     "max_blocks": max block can be produced in blockchain (0 for unlimited)
     "max_nonce": max number for miner to go through and try to produce new valid block
     "difficulty": number of 0 to match in hash to consider the block is valid and added to blockchain
     "tx_waiting_ms": time for miner to wait for new transactions coming
   }
```

### Running web server and miner concurrently

```
cargo run
```

### see all blocks and update the blockchain

```
http://127.0.0.1:8000/blocks
```

### get specific block by index

```
http://127.0.0.1:8000/blocks/get/{index}
```

### see current transaction pool, where miner gets transactions from

```
http://127.0.0.1:8000/tx/pool
```

### create new transaction

```
http://127.0.0.1:8000/tx/new/{from}/{to}/{amount}
```

