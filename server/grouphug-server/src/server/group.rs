//! Logic related to the Groups, the components in charge of managing groups and making sure groups are closed properly when is required.

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use hex::decode as hex_decode;

use bdk::bitcoin::{
    Transaction, 
    consensus::encode::deserialize, 
    consensus::encode::serialize_hex
};
use bdk::electrum_client::{Client, ElectrumApi};


use crate::config::{TESTNET_ELECTRUM_SERVER_ENDPOINT, MAINNET_ELECTRUM_SERVER_ENDPOINT};

/// Time to wait until closing a group if it is not fulfilled (in seconds).
const MAX_TIME: u32 = 43200;

/// Maximum number of participants of each group.
const MAX_SIZE: u8 = 20;

struct Group {
    // Tx data
    id: u32,
    transactions: Vec<(TxIn, TxOut)>,
    transaction_group: Transaction,

    // Control data
    start_time: Instant,
    closed: Arc<AtomicBool>,
}


impl Group {
    fn new(id: u32) -> Self {
        Group {
            id,
            transactions: Vec::new(),
            transaction_group: Transaction {
                version: 2,
                lock_time: 0,
                input: Vec::new(),
                output: Vec::new(),
            },

            start_time: Instant::now(),
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(self: Arc<Mutex<Self>>) {
        /// Starts the lifecicle of a group.
        /// Runs a countdown in seconds until the timeleft reaches 0 or the size of the transaction vector is equal to the maximum group size.
        /// When the group comes to an end, the group transaction is created.
        
        // Clone self with Arc Mutex to share data between threads.
        let this = Arc::clone(&self);

        thread::spawn(move || {
            while Instant::now().duration_since(this.start_time) < MAX_TIME {
                if this.closed.load(Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
            
            // Note: this checks if close_group has already been called by "add_tx" when the transactions.len = MAX_SIZE
            // If not it closes it.
            if !this.closed.load(Ordering::SeqCst) {

                // Lock the mutex in order to writing data
                let mut this = self.lock().unwrap();
                this.close_group();
            }

        });
    }

    pub fn add_tx(&mut self, tx_hex: &str) {
        // tx_hex must be a valid transaction for this group (Checks must be done before)
        // add the transaction to the group

        if self.transactions.len() < MAX_SIZE {
            let tx: Transaction = deserialize(&hex_decode(tx_hex).unwrap()).unwrap();
            self.transactions.push((tx.input[0], tx.output[0]));
        
        } else {
            self.close_group();
        }
        
    }


    fn create_group_transaction(&mut self) {

        // Clean inputs in outputs in case there's some data (should not)
        self.transaction_group.input.clear();
        self.transaction_group.output.clear();

        // add inputs and outputs
        for in_out_tuple in self.transactions {
            self.transaction_group.input.push(in_out_tuple.0);
            self.transaction_group.output.push(in_out_tuple.1);
        }

    }
    

    // Note: Could be changed to send_raw_tx call to the Bitcoin node?
    fn close_group(&mut self) -> bool {
        // Finalize the transaction and send it to the network
        
        // Close the time thread
        self.closed.store(true, Ordering::SeqCst);


        // Create the group transaction
        self.create_group_transaction();
        
        let tx_hex = serialize_hex(&self.transaction_group);
        let tx_bytes = hex_decode(tx_hex).unwrap();

        // Connect to Electrum node
        let client = Client::new(MAINNET_ELECTRUM_SERVER_ENDPOINT).unwrap();
        println!("Connected to the node");

        // broadcast the transaction
        let txid = client.transaction_broadcast_raw(&tx_bytes);

        match txid {
            Ok(Some(id)) => {
                println!("Group {} closed! Transaction broadcasted with TXID: {}", self.id, id);
                return true;
            },
            Ok(None) => {
                println!("Transaction broadcast returned NONE");
                return false;
            }
            Err(_) => {
                println!("There is an error broadcasting the transaction group");
                return false;
            }
    
        }
    }
}