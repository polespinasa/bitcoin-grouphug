//! Logic related to the Groups, the components in charge of managing groups and making sure groups are closed properly when is required.

use std::thread;
use std::time::Duration;


/// Time to wait until closing a group if it is not fulfilled (in seconds).
const TIMELEFT: u32 = 43200;

/// Maximum number of participants of each group.
const MAX_SIZE: u8 = 20;

struct Group {
    id: u32,
    timeleft: u32,
    max_size: u8,
    // transactions: Vec<(inputs, outputs)>, TODO --> Implement logic behind inputs and outputs
    // transaction_group: transaction, TODO --> Implement logic behind transactions
}


impl Group {
    fn new(id: u32) -> Self {
        Group {
            id,
            timeleft: TIMELEFT,
            max_size: MAX_SIZE,
            transactions: Vec::new(),
            transaction_group,

        }
    }

    /// Check if the group must be closed according to the timeleft and max_size condition.
    /// If the group is full or the timeleft reach 0 the group must be closed.
    fn check_if_must_close(&mut self) {
        if self.timeleft <= 0 || self.transactions.len() >= self.max_size {
            true
        }
        self.timeleft -= 1;
        false
    }


    pub fn run(&mut self) {
        /// Starts the lifecicle of a group.
        /// Runs a countdown in seconds until the timeleft reaches 0 or the size of the transaction vector is equal to the maximum group size.
        /// When the group comes to an end, the group transaction is created.
        let handle = thread::spawn(move || {
            while check_if_must_close() == false {
                thread::sleep(Duration::from_secs(1));
            }
            self.create_group_transaction();
        });

        handle.join().unwrap();

        close_group();
    }

    pub fn add_tx(&mut self, tx: transaction) -> bool {
        // Check if tx is valid
        // ...
        // if valid -> add tx input and output to the tupple
        // return true
        // if invalid
        // return false
    }


    fn create_group_transaction(&mut self) {
        // Create group_transaction
        // ...
    }
    

    // Could be changed to send_raw_tx call to the Bitcoin node?
    fn close_group(&self) {
        println!("Group {} closed!", self.id);
    }
}