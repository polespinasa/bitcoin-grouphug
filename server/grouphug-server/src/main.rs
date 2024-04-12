mod utils;
mod config;
mod server;

use std::time::Duration;
use std::thread;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use crate::server::group::Group;
use crate::utils::transactions::validate_tx_query_one_to_one_single_anyone_can_pay;


// GroupHug are from the Group class 
type GroupHug = Group;

// Array with Group list
static GLOBAL_GROUPS: Lazy<Arc<Mutex<Vec<GroupHug>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));


fn handle_addtx(arg: &str, mut stream: TcpStream) {

    let group_index;
    if GLOBAL_GROUPS.lock().unwrap().len() != 1 {
        let first_group = Group::new(0);
        let mut groups = GLOBAL_GROUPS.lock().unwrap();
        groups.push(first_group);
        group_index = groups.len() - 1; // Index of the newly added group
    } else {
        group_index = 0;
    }

    // Before adding the tx ensures that 
    println!("La tx és {}!", arg);

    // Validate that the tx has the correct format
    let (valid, msg) = validate_tx_query_one_to_one_single_anyone_can_pay(arg);
    if valid {
        let mut groups = GLOBAL_GROUPS.lock().unwrap();
        groups[group_index].add_tx(arg);
        stream.write(b"Ok\n").unwrap();
    } else {
        // here should send an error message
        let error_msg = format!("Error: {}\n", msg);
        stream.write(error_msg.as_bytes()).unwrap();
    }

}
/*
fn handle_b() {
    println!("Has rebut la comanda B!");
}

fn handle_c() {
    println!("Has rebut la comanda C!");
}
*/

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    loop {
        let nbytes = stream.read(&mut buffer).unwrap();
        if nbytes == 0 {
            return;
        }

        let command_string = String::from_utf8(buffer[0..nbytes].to_vec()).unwrap();
        let command_parts: Vec<&str> = command_string.trim().split_whitespace().collect();
        if command_parts.len() != 2 {
            println!("Invalid command: {}", command_string);
            continue;
        }
        let (command, arg) = (command_parts[0], command_parts[1]);
        //let command = command_string.trim();
        match command {
            "add_tx" => handle_addtx(arg, stream.try_clone().unwrap()),
            //"B" => handle_b(),
            //"C" => handle_c(),
            _ => println!("Command not known: {}", command),
        }
        //stream.write(&buffer[0..nbytes]).unwrap();
    }
}

fn main() {
    
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
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
    
    
    /* This was the first test
    let tx1: &str = "02000000000101ac47d4326964306dddf3e6225d8667e2f62de9d96fe99b91580f16e29c96695c0000000000fdffffff019e24e51700000000160014ed1fb4a7ca03a2a03f065c0720e4af27c786fb580247304402200a43adf4c17b80453a9679dc4c0587f874c14692461beff5aad12e9cbe78851302204f84c17ccf0b9f77306c42dc8fb4d64f6d407a978d87d4d096845b3611e4698c8321036a65b47b4074d1cdf20cc3ed2a2d440e36f805798267d1c491a01d4efde9fc1100000000";
    let tx2: &str = "02000000000101ac47d4326964306dddf3e6225d8667e2f62de9d96fe99b91580f16e29c96695c0100000000fdffffff0192e0f50500000000160014ed1fb4a7ca03a2a03f065c0720e4af27c786fb580247304402202fac06913e02844b5deb87d7be9be21bbcca09ed2a953a9bdf0e61611682292e022063ecbed39076e0209573d907b8c13f3b4a3d9bb7753c257e8aa32b37855c89c58321036a65b47b4074d1cdf20cc3ed2a2d440e36f805798267d1c491a01d4efde9fc1100000000";
    let tx3: &str = "02000000000101ac47d4326964306dddf3e6225d8667e2f62de9d96fe99b91580f16e29c96695c0200000000fdffffff0192e0f50500000000160014ed1fb4a7ca03a2a03f065c0720e4af27c786fb5802473044022040630b74abe00c72a9c1cad6a725a6fbe2c583f1c1d57909a6d96d2768a62d430220655ceb02a9c3d99fdbfbb569ebe2f6b88dd0f8166f98012621f7d613e94fc4658321036a65b47b4074d1cdf20cc3ed2a2d440e36f805798267d1c491a01d4efde9fc1100000000";
    let tx4: &str = "02000000000101ac47d4326964306dddf3e6225d8667e2f62de9d96fe99b91580f16e29c96695c0300000000fdffffff0192e0f50500000000160014ed1fb4a7ca03a2a03f065c0720e4af27c786fb5802473044022008eecc3947b7ee5be8ae55042d51bcec6a0e8a062e800d898456c8fb6ce7ad1302205de58f338d89310946ff8cb0e588b0f1cd5b3b6d7cf697c6700d537f339663968321036a65b47b4074d1cdf20cc3ed2a2d440e36f805798267d1c491a01d4efde9fc1100000000";
    let tx5: &str = "02000000000101ac47d4326964306dddf3e6225d8667e2f62de9d96fe99b91580f16e29c96695c0400000000fdffffff0192e0f50500000000160014ed1fb4a7ca03a2a03f065c0720e4af27c786fb58024730440220545ebb9ead993aa556ee0fc47069a374edffbbb6354aa808095302807bab607c022028d6acfc9f8814d94fe1ee0432ecdc22d7c8b16a83fbfcd232775e0767f13f368321036a65b47b4074d1cdf20cc3ed2a2d440e36f805798267d1c491a01d4efde9fc1100000000";
    let tx6: &str = "02000000000101ac47d4326964306dddf3e6225d8667e2f62de9d96fe99b91580f16e29c96695c0500000000fdffffff0192e0f50500000000160014ed1fb4a7ca03a2a03f065c0720e4af27c786fb5802473044022047ea368a3b4f117fcbaddb604e47b0bbb3f89c7543186be18c2168b321fd2898022049aad27921872a4160ede19dfbe2df013e59330ccd6fd3db0838dabb66bbef0a8321036a65b47b4074d1cdf20cc3ed2a2d440e36f805798267d1c491a01d4efde9fc1100000000";
    let tx7: &str = "02000000000101ac47d4326964306dddf3e6225d8667e2f62de9d96fe99b91580f16e29c96695c0600000000fdffffff0192e0f50500000000160014ed1fb4a7ca03a2a03f065c0720e4af27c786fb580247304402204726f4e04e403af7e4eaa20efc623fd95b95f479ecd1ba88a3a1ff3dbc6755870220725c426d79811782da6ba90495bb36a809ab4356ec3d941896befc6387a4c9118321036a65b47b4074d1cdf20cc3ed2a2d440e36f805798267d1c491a01d4efde9fc1100000000";
    let tx8: &str = "02000000000101b823b3fdd61c4d9bb4fc29b1c4950da3d04daeb4f0a44c5a1b669d88248fb8d80000000000fdffffff0186df130000000000160014c5b8f3f58e7062507b52b47f9111d13790c35895024730440220502ea9ea5d0cd1517999d30d3acc2448a16bea631091aed7c00293d228a9629802207de829fa0f4cf93e96f61975fd0cbbc95300f5fdf4b8e929cdf4b7866508638a8321021a35b89c057789b85b7e4fdcbbaf1dc80466a5cb493548f170f8affc3ceb0d0b00000000";

    first_group.add_tx(tx1);
    first_group.add_tx(tx2);
    first_group.add_tx(tx3);
    first_group.add_tx(tx4);
    first_group.add_tx(tx5);
    first_group.add_tx(tx6);
    first_group.add_tx(tx7);
    first_group.add_tx(tx8);

    //result: efb8494cbacd70d5d55ac2fb0194777fec9d1e58de96a4754f7c31b5fa807c2a
    */

    println!("Wellcome to the bitcoin group hug!!");
}
