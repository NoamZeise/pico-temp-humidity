extern crate serialport;

use std::env;
use pico_th_collector::{get_readings, save_data_as_csv};

fn main() {
    let mut port : Option<String> = None;
    let mut save_file : Option<String> = None;
    for (i, arg) in env::args().enumerate() {
        match i {
            1 => port = Some(arg),
            2 => save_file = Some(arg),
            _ => ()
        }
    }
    let port = match port {
        Some(p) => p,
        None => {
            println!(
"No port argument supplied
ensure port is COM[x] on windows or /dev/rfcomm[x]
on linux,corresponding to the paired bluetooth module (HC-05)");
            failed_msg();
            return
        }
    };
    let save_file = match save_file {
        Some(p) => p,
        None => {
            println!("No save file argument supplied!\n");
            failed_msg();
            return
        }
    };

    let readings = match get_readings(&port) {
        Ok(k) => k,
        Err(e) => {
            println!("Error:{}", e);
            return
        }
    };

    save_data_as_csv(readings, &save_file);
}

fn failed_msg() {
    println!("expected args:\n  [port] [save file]");
}
