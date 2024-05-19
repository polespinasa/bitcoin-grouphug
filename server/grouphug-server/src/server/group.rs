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
use bdk::electrum_client::{Client, ConfigBuilder, ElectrumApi};
use bdk::blockchain::{ElectrumBlockchain, GetTx};
use chrono::Utc;

pub struct Group {
    pub fee_rate: f32,
    pub timestamp: i64,
    transactions: Vec<(TxIn, TxOut)>,
    transaction_group: Transaction,

}


impl Group {
    pub fn new(fee_rate: f32) -> Self {
        Group {
            fee_rate,
            timestamp: Utc::now().timestamp(),
            transactions: Vec::new(),
            transaction_group: Transaction {
                version: 2,
                lock_time: LockTime::from_height(0).unwrap(),
                input: Vec::new(),
                output: Vec::new(),
            },

        }
    }

    pub fn get_num_transactions(&self) -> usize {
        // Return number of transactions in the group
        return self.transactions.len()
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

        for i in 0..tx.input.len() {
            self.transactions.push((tx.input[i].clone(), tx.output[i].clone()));
        }

        println!("Tx {} added to group with fee_rate {}sat/vB", tx.txid(), self.fee_rate);

        // Check if the group should be closed according to the MAX_SIZE limit established in config file
        if self.transactions.len() >= crate::CONFIG.group.max_size {
            return self.close_group();
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
    

    pub fn close_group(&mut self) -> bool {
        // Finalize the transaction and send it to the network
    
        // Connect to Electrum node
        let config = ConfigBuilder::new().validate_domain(crate::CONFIG.electrum.certificate_validation).build();
        let client = Client::from_config(&crate::CONFIG.electrum.endpoint, config.clone()).unwrap();
        let blockchain = ElectrumBlockchain::from(client);
        
        // Check that the transactions included in the group have not been already spent
        // If they've remove them from the group and don't close it
        let mut i = 0;
        while i != self.transactions.len() {
            let in_out_tuple = &self.transactions[i];
            let outpoint = in_out_tuple.0.clone().previous_output;
            let tx_result = blockchain.get_tx(&outpoint.txid);
            match tx_result {
                Ok(Some(tx)) => {
                    // validate if the output has been spent
                    let utxo_script_pubkey = &tx.output[outpoint.vout as usize].script_pubkey;
                    let utxo_list = blockchain.script_list_unspent(&utxo_script_pubkey);
                    match utxo_list {
                        Ok(returned_utxo_list) => {
                            if returned_utxo_list.len() > 0 {
                                i += 1;
                            }
                            else {
                                eprintln!("Double spending detected on a group, deleting that transaction...");
                                self.transactions.remove(i);
                                return false;
                            }
                        },
                        Err(_e) => {
                            eprintln!("Error querying for the UTXO");
                            return false;
                        }
                    }
                },
                Ok(None) => {
                    eprintln!("Petition succeed but no tx was returned");
                    return false;
                },
                Err(_e) => {
                    println!("Could not retrieve previous transaction");
                    return false;
                }
            }
        }
        

        // Create the group transaction
        self.create_group_transaction();
        
        let tx_hex = serialize_hex(&self.transaction_group);
        println!("Group transaction: \n");
        println!("{:?}", tx_hex);

        let tx_bytes = hex_decode(tx_hex).unwrap();

        // broadcast the transaction
        // There's a issue with client 1 here... TODO FIX
        let client2 = Client::from_config(&crate::CONFIG.electrum.endpoint, config.clone()).unwrap();
        let txid = client2.transaction_broadcast_raw(&tx_bytes);

        match txid {
            Ok(id) => {
                println!("Group {}sat/vb closed! Transaction broadcasted with TXID: {}", self.fee_rate, id);
                return true;
            },
            Err(e) => {
                eprintln!("There is an error broadcasting the transaction group: {:?}", e);
                return false;
            }
    
        }
    }
}