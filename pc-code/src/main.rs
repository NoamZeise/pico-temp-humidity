extern crate serialport;

use std::env;
use pico_th_collector::{get_readings, save_data};

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
    let port = port.unwrap_or_else(||
        {
            failed_msg();
            panic!("No port argument supplied")
        });
    let save_file = save_file.unwrap_or_else(||
        {
            failed_msg();
            panic!("No save file argument supplied")
        });

    save_data(get_readings(&port), &save_file);
}

fn failed_msg() {
    println!("expected args:\n  [port] [save file]");
}
