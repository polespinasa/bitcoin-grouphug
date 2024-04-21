//! Functions related to the transactions validation and manipulation.

use bdk::bitcoin::{
    OutPoint,
    Transaction,
    consensus::encode::deserialize,
    blockdata::locktime::absolute::{Height, Time}};

use bdk::blockchain::{ElectrumBlockchain, GetTx};
use bdk::electrum_client::{Client, ElectrumApi};
use hex::decode as hex_decode;


pub fn which_network(tx: &Transaction) -> bool {

    // Take previous UTXO
    let tx_id = tx.input[0].previous_output.txid;

    // Test mainnet
    let client_mainnet = Client::new(&crate::CONFIG.electrum.endpoint).unwrap();
    let blockchain_mainnet = ElectrumBlockchain::from(client_mainnet);
    
    
    let tx_result_mainnet = blockchain_mainnet.get_tx(&tx_id);
    match tx_result_mainnet {
        Ok(Some(_tx)) => {
            return true
        },
        Ok(None) => (),
        Err(_) => (),
    }
    return false;
}

pub fn get_previous_utxo_value(utxo: OutPoint) -> f32 {
    // Given an input from a certain transaction returns the value of the pointed UTXO.
    // If no UTXO is recieved back, the value returned is 0.

    // Connect to Electrum node
    let client = Client::new(&crate::CONFIG.electrum.endpoint).unwrap();
    
    let blockchain = ElectrumBlockchain::from(client);

    let tx_result = blockchain.get_tx(&utxo.txid);

    match tx_result {
        Ok(Some(tx)) => {
            return tx.output[utxo.vout as usize].value as f32;
        },
        Ok(None) => {
            println!("Previous transaction query returned NONE");
            return 0.0;
        }
        Err(erro) => {
            println!("{}", erro);
            println!("There is an error retrieving previous transaction");
            return 0.0;
        }

    }
}

pub fn previous_utxo_spent(tx: &Transaction) -> bool {
    // Validates that the utxo pointed to by the transaction input has not been spent.

    // Connect to Electrum node
    let client = Client::new(&crate::CONFIG.electrum.endpoint).unwrap();
    
    let blockchain = ElectrumBlockchain::from(client);

    // Get the previous transaction from the input
    let outpoint = tx.input[0].previous_output;
    let tx_result = blockchain.get_tx(&outpoint.txid);

    match tx_result {
        Ok(Some(tx)) => {

            // validate if the output has been spent
            let utxo_script_pubkey = &tx.output[outpoint.vout as usize].script_pubkey;
            let utxo_list = blockchain.script_list_unspent(&utxo_script_pubkey);

            match utxo_list {
                Ok(returned_utxo_list) => {
                    if returned_utxo_list.len() > 0 {
                        return true;
                    }
                    else {
                        println!("Transaction already spent");
                        return false;
                    }
                },
                Err(_e) => {
                    println!("Error querying for the UTXO");
                    return false;
                }
            }
        },
        Ok(None) => {
            print!("Petition succeed but no tx was returned");
            return false;
        },
        Err(_e) => {
            println!("Could not retrieve previous transaction");
            return false;
        }
    }
}

pub fn check_absolute_locktime(tx: &Transaction) -> bool {
    // Return true or false depending if the absolute locktime is 0.
    let height_expected = Height::from_consensus(0).unwrap();
    let time_expected = Time::MIN;
    return tx.is_absolute_timelock_satisfied(height_expected, time_expected);
}

pub fn check_dust_limit(tx: &Transaction) -> bool {
    // Return true or false if the tx value is >= than the DUST_LIMIT.
    return tx.output[0].value >= crate::CONFIG.dust.limit;
}

pub fn check_tx_version(tx: &Transaction) -> bool {
    // Return ture or false if the tx version is 2
    return tx.version == 2;
}

pub fn get_num_inputs_and_outputs(tx: &Transaction) -> (usize, usize) {
    // Return the number of inputs and outputs from a given transaction in a tuple
    return (tx.input.len(), tx.output.len());
}

pub fn check_sighash_single_anyone_can_pay(tx: &Transaction) -> bool {
    // Ensure that the signature is using SIGHASH_SINGLE|ANYONECANPAY
    // Script must be simple P2WPKH (witness: <signature> <pubkey>)

    if tx.input[0].witness.len() != 2 {
        return false;
    }

    let input_query = tx.input[0].witness.to_vec()[0].clone();

    match input_query.last() {
        Some(input) => {
            // 131 decimal representation of 0x83 designated to SIGHASH_SINGLE | ANYONECANPAY
            if *input != 131 as u8{
                return false;
            }
        },
        None => {
            // There's no witness
            return false;
        }
    }

    return true;
}

pub fn validate_tx_query_one_to_one_single_anyone_can_pay(tx_hex: &str ) -> (bool, String, f32) {
    // Validate that a given transaction (in hex) is valid according to the rules.
    // Returns true if the tx is valid. String with the error message if any and a f32 with the fee_rate of the transaction    
    // Rules:
    //  - Should only be 1 input.
    //  - Should only be 1 output.
    //  - The input cannot be spent before must be and UTXO.
    //  - Signature must be SIGHASH_SINGLE | ANYONECANPAY.
    //  - Should have absolute locktime to 0.
    //  - Fee rate should be bigger than 1.01sat/vb
    
    
    let mut real_fee_rate: f32 = 0.0;
    
    let tx_hex_decoded = match hex_decode(tx_hex) {
        Ok(decoded) => decoded,
        Err(_) => return (false, String::from("Error decoding hex"), real_fee_rate),
    };
    let tx: Transaction = match deserialize(&tx_hex_decoded) {
        Ok(transaction) => transaction,
        Err(_) => return (false, String::from("Error deserializing transaction"), real_fee_rate),
    };
    
    // Check that the transaction belongs to the specified network
    let network: bool = which_network(&tx);
    if !network {
        let msg = format!("The tx you provided is not from {} network", &crate::CONFIG.network.name);
        return (false, msg, real_fee_rate);
    }
    
    // Check that there is only one input
    let num_inputs_and_outputs: (usize, usize) = get_num_inputs_and_outputs(&tx);
    if  num_inputs_and_outputs != (1,1) {
        let msg = format!("Number of inputs and outputs must be 1. Inputs = {} | Outputs = {}", num_inputs_and_outputs.0, num_inputs_and_outputs.1);
        return (false, msg, real_fee_rate);
    }
    
    // Check that the absolute lock time is disabled or set to 0
    let abs_lock_time: bool = check_absolute_locktime(&tx);
    if !abs_lock_time {
        let msg = String::from("Absolute locktime is not 0");
        return (false,msg, real_fee_rate);
    }

    // Check that the transaction value is over the dust limit specified in the config file
    let dust_limit_valid: bool = check_dust_limit(&tx);
    if !dust_limit_valid {
        let msg = format!("The transaction value is under the dust limit {}", &crate::CONFIG.dust.limit);
        return (false,msg, real_fee_rate);
    }

    // Check that the transaction version is v2
    let tx_version_correct: bool = check_tx_version(&tx);
    if !tx_version_correct{
        let msg = String::from("Tx version is not 2");
        return (false, msg, real_fee_rate);
    }

    
    // Check that previous utxo value is not 0
    // Aka is not an op_return
    let previous_utxo_value: f32 = get_previous_utxo_value(tx.input[0].previous_output);
    if previous_utxo_value == 0.0 {
        let msg = String::from("There's an error loading the previous utxo value");
        return (false,msg, real_fee_rate);
    } else{
        real_fee_rate = (previous_utxo_value - tx.output[0].value as f32)/tx.vsize() as f32;
    }

    // Check that the fee rate is not under 1sat/vb
    if real_fee_rate <= 1.01 {
        let msg = format!("Fee bellow 1 sat/vB. Fee rate found {}sat/vB", real_fee_rate);
        return (false,msg, real_fee_rate);
    }
    

    // Check that the signature type is SIGHASH_SINGLE |ANYONECANPAY
    if !check_sighash_single_anyone_can_pay(&tx) {
        let msg = String::from("Wrong sighash used");
        return (false,msg, real_fee_rate);
    }

    // Check if there's a double spending attempt
    if !previous_utxo_spent(&tx) {
        let msg = String::from("Double spending detected");
        return (false,msg, real_fee_rate);
    }

    
    return (true, String::from("Ok"), real_fee_rate);

}