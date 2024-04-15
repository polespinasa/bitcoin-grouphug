//! Logic related to the Groups, the components in charge of managing groups and making sure groups are closed properly when is required.

use hex::decode as hex_decode;

use bdk::bitcoin::{
    Transaction,
    TxIn,
    TxOut,
    blockdata::locktime::absolute::LockTime,
    consensus::encode::deserialize, 
    consensus::encode::serialize_hex
};
use bdk::electrum_client::{Client, ElectrumApi};


use crate::config::{
    ELECTRUM_ENDPOINT,
    MAX_SIZE,
    //MAX_TIME
};

pub struct Group {
    pub fee_rate: f32,
    transactions: Vec<(TxIn, TxOut)>,
    transaction_group: Transaction,

}


impl Group {
    pub fn new(fee_rate: f32) -> Self {
        Group {
            fee_rate,
            transactions: Vec::new(),
            transaction_group: Transaction {
                version: 2,
                lock_time: LockTime::from_height(0).unwrap(),
                input: Vec::new(),
                output: Vec::new(),
            },

        }
    }

    pub fn contains_txin(&self, txin: &TxIn) -> bool {
        // Return true or false if the given tx input is already in this group
        self.transactions.iter().any(|(t, _)| t.previous_output == txin.previous_output)
    }
    

    pub fn add_tx(&mut self, tx_hex: &str) -> bool {
        // tx_hex must be a valid transaction for this group (Checks must be done before)
        // add the transaction to the group
        // return true or false depending if the group has been closed after adding the new transaction

        let tx: Transaction = deserialize(&hex_decode(tx_hex).unwrap()).unwrap();

        self.transactions.push((tx.input[0].clone(), tx.output[0].clone()));

        // Check if the group should be closed according to the MAX_SIZE limit established in config file
        if self.transactions.len() == MAX_SIZE {
            self.close_group();
            return true;
        }
        return false;
    }


    fn create_group_transaction(&mut self) {
        // Creates the final group transaction ready to be broadcasted

        // Clean inputs in outputs in case there's some data (should not)
        self.transaction_group.input.clear();
        self.transaction_group.output.clear();

        // add inputs and outputs
        for in_out_tuple in &self.transactions {
            self.transaction_group.input.push(in_out_tuple.0.clone());
            self.transaction_group.output.push(in_out_tuple.1.clone());
        }
    }
    

    // Note: Could be changed to send_raw_tx call to the Bitcoin node?
    fn close_group(&mut self) -> bool {
        // Finalize the transaction and send it to the network
    
        // Create the group transaction
        self.create_group_transaction();
        
        let tx_hex = serialize_hex(&self.transaction_group);
        println!("Group transaction: \n");
        println!("{:?}", tx_hex);

        let tx_bytes = hex_decode(tx_hex).unwrap();

        // Connect to Electrum node
        let client = Client::new(ELECTRUM_ENDPOINT.get().unwrap()).unwrap();

        // broadcast the transaction
        let txid = client.transaction_broadcast_raw(&tx_bytes);

        match txid {
            Ok(id) => {
                println!("Group {}sat/vb closed! Transaction broadcasted with TXID: {}", self.fee_rate, id);
                return true;
            },
            Err(e) => {
                println!("There is an error broadcasting the transaction group: {:?}", e);
                return false;
            }
    
        }
    }
}