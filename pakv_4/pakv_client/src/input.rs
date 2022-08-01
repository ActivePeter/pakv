use std::io;
use std::io::Write;
use std::sync;
use crate::client::PakVClient;

pub fn input_wait(){

    loop {
        // Print command prompt and get command
        print!("> ");
        io::stdout().flush().expect("Couldn't flush stdout");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error reading input.");
        PakVClient::new().oneshot(input);
    }
}
