// Local imports
mod utils;
mod config;
mod server;
use crate::utils::transactions::validate_tx_query_one_to_one_single_anyone_can_pay;
use crate::config::Config;
use crate::server::group::Group;

// External libraries
use std::{
    thread,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str,
    fs,
    env,
    sync::{Arc, Mutex},
};
use once_cell::sync::Lazy;
use hex::decode as hex_decode;
use bdk::bitcoin::{Transaction,consensus::encode::deserialize};

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    
    let args: Vec<String> = env::args().collect();

    // Default Config.toml is same dir as the bin
    let default_path = "Config.toml";

    if args.len() > 2 {
        eprintln!("Only 1 argument accepted");
        std::process::exit(1);
    }

    // If there's an argumment try to use it as config path
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        default_path
    };

    let contents = fs::read_to_string(config_path)
        .expect("Something went wrong reading the file");

    let config: Config = toml::from_str(&contents)
        .expect("Unable to parse the toml file");

    config
});


// Array with Group list
type GroupHug = Group;
static GLOBAL_GROUPS: Lazy<Arc<Mutex<Vec<GroupHug>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

fn check_double_spending_other_group(tx_hex: &str) -> (bool, String) {
    // Check if an input from a transaction is already duplicated on another group
    // Return true if cheating is detected
    /* REPLACE BY FEE NOT IMPLEMENTED ON GROUPS YET */
    
    
    // Decode the transaction received
    let tx_hex_decoded = match hex_decode(tx_hex) {
        Ok(decoded) => decoded,
        Err(_) => return (true, String::from("Error decoding hex")),
    };
    let tx: Transaction = match deserialize(&tx_hex_decoded) {
        Ok(transaction) => transaction,
        Err(_) => return (false, String::from("Error deserializing transaction")),
    };

    
    // Lock the global groups and iterate over them
    let groups = GLOBAL_GROUPS.lock().unwrap();
    
    for txin in tx.input.iter(){
        for group in groups.iter() {
            // Checks if a tx input is in the group
            if group.contains_txin(&txin) {
                eprintln!("Transaction was rejected, Error: transaction input is already in a group\n");
                return (true, String::from("Transaction input is already in a group"));
            }
        }
    }
    

    return (false, String::from("Ok\n"));

}

fn handle_addtx(transaction: &str, mut stream: TcpStream) {

    // Validate that the tx has the correct format and satisfies all the rules
    let (valid, msg, fee_rate) = validate_tx_query_one_to_one_single_anyone_can_pay(transaction);

    println!("Client {} sent a new raw transaction: {}",stream.peer_addr().unwrap(), transaction);

    if !valid {
        // should send an error message as the transaction has an invalid format or does not match some rule
        let error_msg = format!("Error: {}\n", msg);
        eprintln!("Transaction was rejected, {}\n", error_msg);
        stream.write(error_msg.as_bytes()).unwrap();
        return
    }


    
    let (double_spend, msg) = check_double_spending_other_group(transaction);
    if double_spend {
        // should send an error as we detected that the tx input has been already added to another group
        let error_msg = format!("Error: {}\n", msg);
        stream.write(error_msg.as_bytes()).unwrap();
        return
    }

    // Calculate the group fee rate.
    let expected_group_fee = ((fee_rate / &crate::CONFIG.fee.range).floor() * &crate::CONFIG.fee.range) as f32;

    // Unlock the GLOBAL_GROUPS variable
    let mut groups = GLOBAL_GROUPS.lock().unwrap();

    // Search for the group corresponing to the transaction fee rate
    let group = groups.iter_mut().find(|g| g.fee_rate == expected_group_fee);

    let close_group;
    match group {
        Some(group) => {
            // If some then the group already exist so we add the tx to that group
            close_group = group.add_tx(transaction);
        },
        None => {
            // If none then there is no group for this fee rate so we create one
            let mut new_group = Group::new(expected_group_fee);
            println!("New group created with fee_rate {}sat/vB", new_group.fee_rate);
            close_group = new_group.add_tx(transaction);
            groups.push(new_group);
        }
    }

    if close_group {
        // If the group has been closed during the add_tx function we delete it from the groups vector
        groups.retain(|g| g.fee_rate != expected_group_fee);
    }
    

    // Send an OK message if the tx was added successfuly
    stream.write(msg.as_bytes()).unwrap();

    return;
}


fn handle_client(mut stream: TcpStream) {

    println!("New user connected: {}\n", stream.peer_addr().unwrap());

    // send the network configuration
    // TODO -> Find a way to ask the electrum server what network is running
    if &crate::CONFIG.network.name == "testnet" {
        stream.write(b"TESTNET\n").unwrap();
    }
    else if &crate::CONFIG.network.name == "mainnet" {
        stream.write(b"MAINNET\n").unwrap();
    }
    
    // 100KB size for large transactions
    let mut buffer = [0; 100*1024]; 
    loop {
        let nbytes = stream.read(&mut buffer).unwrap();
        if nbytes == 0 {
            return;
        }

        let command_string = match String::from_utf8(buffer[0..nbytes].to_vec()) {
            Ok(s) => s,
            Err(_e) => {
                // If error user has disconnected
                println!("Client {} disconnected\n", stream.peer_addr().unwrap());
                return;
            },
        };
        


        let command_parts: Vec<&str> = command_string.trim().split_whitespace().collect();
        
        if command_parts.len() != 2 {
            // If there's more than two arguments on the call something is worng.
            // Expected format: "add_tx raw_tx_data"
            eprintln!("Client {} sent a command with wrong number of arguments: {}\n", stream.peer_addr().unwrap(), command_string.trim());
            stream.write(b"Two arguments are expected\n").unwrap();
            continue;
        }
        let (command, arg) = (command_parts[0], command_parts[1]);
        
        match command {
            // This allows to add more commands in the future
            "add_tx" => handle_addtx(arg, stream.try_clone().unwrap()),
            _ => {
                eprintln!("Client {} sent an unknown command: {}\n", stream.peer_addr().unwrap(), command);
                stream.write(b"Unknown command sent\n").unwrap();
            },
        }
    }
}



fn main() {
           
    // Fromat endpoint data from config file
    let endpoint: String = format!("{}:{}", &crate::CONFIG.server.ip, &crate::CONFIG.server.port);
    
    let listener = TcpListener::bind(endpoint.clone()).unwrap();

    println!("Server running on {}", endpoint);
    for stream in listener.incoming(){
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Unable to connect: {}", e);
            }
        }
    }
}
