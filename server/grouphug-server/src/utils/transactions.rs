//! Functions related to the transactions validation and manipulation.


use bdk::bitcoin::OutPoint;
use bdk::bitcoin::{
    Transaction,
    consensus::encode::deserialize,
    blockdata::locktime::absolute::{Height, Time}};

use bdk::blockchain::{ElectrumBlockchain, GetTx};
use bdk::electrum_client::{Client, ElectrumApi};
use hex::decode as hex_decode;

use crate::config::{TESTNET_ELECTRUM_SERVER_ENDPOINT,
    MAINNET_ELECTRUM_SERVER_ENDPOINT,
    ELECTRUM_ENDPOINT,
    DUST_LIMIT,
    NETWORK
};


pub fn which_network(tx: &Transaction) -> &str {

    // Take previous UTXO
    let tx_id = tx.input[0].previous_output.txid;

    println!("Tx id {}", tx_id.to_string());

    // Test mainnet
    let client_mainnet = Client::new(MAINNET_ELECTRUM_SERVER_ENDPOINT).unwrap();
    let blockchain_mainnet = ElectrumBlockchain::from(client_mainnet);
    
    
    let tx_result_mainnet = blockchain_mainnet.get_tx(&tx_id);
    match tx_result_mainnet {
        Ok(Some(_tx)) => {
            return "MAINNET"
        },
        Ok(None) => (),
        Err(_) => (),
    }
    
    // Test testnet
    let client_testnet = Client::new(TESTNET_ELECTRUM_SERVER_ENDPOINT).unwrap();
    let blockchain_testnet = ElectrumBlockchain::from(client_testnet);
    let tx_result_testnet = blockchain_testnet.get_tx(&tx_id);

    match tx_result_testnet {
        Ok(Some(_tx)) => {
            return "TESTNET";
        },
        Ok(None) => (),
        Err(_) => (),
    }

    return "UNKNOWN";
}

pub fn get_previous_utxo_value(utxo: OutPoint) -> f32 {
    // Given an input from a certain transaction returns the value of the pointed UTXO.
    // If no UTXO is recieved back, the value returned is 0.

    // Connect to Electrum node
    let client = Client::new(ELECTRUM_ENDPOINT.get().unwrap()).unwrap();
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
    let client = Client::new(ELECTRUM_ENDPOINT.get().unwrap()).unwrap();
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
    return tx.output[0].value >= DUST_LIMIT;
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
    let network: &str = which_network(&tx);
    if network != *NETWORK.get().unwrap() {
        let msg = format!("The grouphug network is {} and you sent a transaction from the {} network", *NETWORK.get().unwrap(), network);
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
        let msg = format!("The transaction value is under the dust limit {}", DUST_LIMIT);
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
        let msg = format!("Fee bellow 1 sat/vb. Fee rate found {}", real_fee_rate);
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



// TESTS MAY NOT WORK AS THEY ARE NOT UPDATED
#[cfg(test)]

mod tests {

    use crate::utils::transactions;
    
    #[test]
    fn test_validate_tx_query_utxo_wrong_sighash() {
        let fee_rate: f32 = 1.0;
    
        //tx should be rejected because of wrong sighash type
        let tx_hex = "0200000000010120855900d4b1009d46b257be2a1b773b154364d21f4e358b8b3d2d617f5d44a00200000000fdffffff0192e0f5050000000016001474ea10df0c2455406b686cd7060bad71feb08b740247304402206262868854e24b4d99a9da929e0955555218ea120b52a35bc8435820b7d7c14c0220641129e5f855a1eb8f659cb487d0e509b1cc664a774fb2322d6e598809f5430b012103bbf8c79c3158a1cb32ab3dcffcb2c0e0a677fc600d7ce8e0fb7ff5294ce2574e80552700";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), false);
    }

    #[test]
    fn validate_tx_query_2_outputs() {
        let fee_rate: f32 = 1.0;

        //tx should be rejected because has 2 outputs
        let tx_hex = "0200000000010120855900d4b1009d46b257be2a1b773b154364d21f4e358b8b3d2d617f5d44a00200000000fdffffff02404b4c000000000016001474ea10df0c2455406b686cd7060bad71feb08b743395a905000000001600142a9e8c87018f003bddc8de007109eaef3295384d0247304402207abd64a3c565f070ebbb560c8134e0e57530f7461e3ff70b7a378dbc59cec07102207f30dbbe33bf374e35afb16cd29e91b5dc855341f384bee4fba5f35fe89d348f832103bbf8c79c3158a1cb32ab3dcffcb2c0e0a677fc600d7ce8e0fb7ff5294ce2574e80552700";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex),false);
    }

    #[test]
    fn validate_tx_query_2_inputs() {
        let fee_rate: f32 = 1.0;

        //tx should be rejected because has 2 inputs
        let tx_hex = "0200000000010220855900d4b1009d46b257be2a1b773b154364d21f4e358b8b3d2d617f5d44a00000000000fdffffff20855900d4b1009d46b257be2a1b773b154364d21f4e358b8b3d2d617f5d44a00100000000fdffffff014ec1eb0b0000000016001474ea10df0c2455406b686cd7060bad71feb08b74024730440220573432bfdbeaf51478a9792aed4865312458c583eceacadbd26e02b97f04889102204c0b9037e1085dd4d16c1cb3898cc9e34f68e9bc0c2ad826213585d632091b98832103bbf8c79c3158a1cb32ab3dcffcb2c0e0a677fc600d7ce8e0fb7ff5294ce2574e0247304402207dfe8f0686eefbd2d8619a426f6715ac9dbb606f4525e9bb4d02b78338461596022074ef896493baddbfbe8980348124f696e23e66b678e366742fc8f1e04bcfccef832103bbf8c79c3158a1cb32ab3dcffcb2c0e0a677fc600d7ce8e0fb7ff5294ce2574e80552700";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), false);
    }

     #[test]
    fn validate_tx_query_valid_tx() {
        let fee_rate: f32 = 1.5;

        //tx for this tust must satisfy all requirements
        let tx_hex = "0200000000010136740240418792cf35e8dea54f9ec215170594ecef73740bd001392a7b464d110100000000fdffffff0129260000000000001600149664e4a54e7f04f09799d2e61268057a033876780247304402204bb811853c6e0f8e49b0bab3ced01da988afc3a08bf127962ab24129be555f1c0220026f413918a7f9fdc02968271e7c730cadb5a3a5476505bc106a87ff1e1f23c3832102fc889ef1d04e7c489f225a718b3742893d88e1e3f6662c1e5e84c7489252968c80552700";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), true);
    }
    

    #[test]
    fn test_validate_tx_query_fee_to_low() {
        let fee_rate: f32 = 7.0;
    
        //tx should be rejected as real fee is below the declarated one.
        let tx_hex = "0200000000010120855900d4b1009d46b257be2a1b773b154364d21f4e358b8b3d2d617f5d44a00000000000fdffffff0161dff5050000000016001474ea10df0c2455406b686cd7060bad71feb08b7402473044022072f5f0603a6229efc0bffb4922c44d01772325577f73b3a76935c23b6947c8e4022067ffc97c49605ae77e0c320a598ab254dee35f87aff16d05858eeda4ee325a64012103bbf8c79c3158a1cb32ab3dcffcb2c0e0a677fc600d7ce8e0fb7ff5294ce2574e80552700";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), false);
    }

    #[test]
    fn test_validate_tx_query_double_spending() {
        let fee_rate: f32 = 1.0;
    
        //tx should be rejected as real fee is below the declarated one.
        let tx_hex = "0200000000010120855900d4b1009d46b257be2a1b773b154364d21f4e358b8b3d2d617f5d44a00300000000fdffffff01c8b4c6290000000016001474ea10df0c2455406b686cd7060bad71feb08b7402473044022001ff2702495ff5ed0b73a178b8e9f84eccc8475e0c5c8a4306abbd56cbcf91e30220465844718edb42df0354cca7bbe91e9798eaedbc8c9dc18322a84cfb77dbd6bc8321028f1b8a4db265de2e99e3ba9575d8b572c8cb85bb12697f7e26a7288b9a4b13ac80552700";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), false);
    }

}