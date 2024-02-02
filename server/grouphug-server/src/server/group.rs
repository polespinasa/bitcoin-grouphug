//! Logic related to the Groups, the components in charge of managing groups and making sure groups are closed properly when is required.

use std::thread;
use std::time::Duration;
use hex::decode as hex_decode;

use bdk::bitcoin::{
    Transaction, 
    consensus::encode::deserialize, 
    consensus::encode::serialize_hex
};
use bdk::electrum_client::{Client, ElectrumApi};


/// Time to wait until closing a group if it is not fulfilled (in seconds).
const TIMELEFT: u32 = 43200;

/// Maximum number of participants of each group.
const MAX_SIZE: u8 = 20;

struct Group {
    id: u32,
    timeleft: u32,
    max_size: u8,
    transactions: Vec<(TxIn, TxOut)>,
    transaction_group: Transaction,
}


impl Group {
    fn new(id: u32) -> Self {
        Group {
            id,
            timeleft: TIMELEFT,
            max_size: MAX_SIZE,
            transactions: Vec::new(),
            transaction_group: Transaction {
                version: 2,
                lock_time: 0,
                input: Vec::new(),
                output: Vec::new(),
            },

        }
    }

    /// Check if the group must be closed according to the timeleft and max_size condition.
    /// If the group is full or the timeleft reach 0 the group must be closed.
    fn check_if_must_close(&mut self) -> bool {
        if self.timeleft <= 0 || self.transactions.len() >= self.max_size {
            return true;
        }
        self.timeleft -= 1;
        return false;
    }


    pub fn run(&mut self) {
        /// Starts the lifecicle of a group.
        /// Runs a countdown in seconds until the timeleft reaches 0 or the size of the transaction vector is equal to the maximum group size.
        /// When the group comes to an end, the group transaction is created.
        
        // this will not work, need to investigate how to work with threads and share data
        let handle = thread::spawn(move || {
            while self.check_if_must_close() == false {
                thread::sleep(Duration::from_secs(1));
            }
        });

        handle.join().unwrap();

        self.close_group();
    }

    pub fn add_tx(&mut self, tx_hex: &str) {
        // tx_hex must be a valid transaction for this group (Checks must be done before)
        // add the transaction to the group

        let tx: Transaction = deserialize(&hex_decode(tx_hex).unwrap()).unwrap();
        self.transactions.push((tx.input[0], tx.output[0]));
        
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
    fn close_group(&self) -> bool {
        // Finalize the transaction and send it to the network
        
        self.create_group_transaction();
        
        let tx_hex = serialize_hex(&self.transaction_group);
        let tx_bytes = hex_decode(tx_hex).unwrap();

        // Connect to Electrum node
        let client = Client::new("umbrel.local:50001").unwrap();
        
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