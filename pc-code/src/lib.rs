extern crate serialport;

use std::time::Duration;
use std::fs::File;
use std::path::Path;
use std::io::{Write, Error};

fn open_port_with_device(port: &str, read_delay: u64) -> Result<Box<dyn serialport::SerialPort>, String> {
    match serialport::new(port, 9600)
        .timeout(Duration::from_millis(read_delay))
        .open() {
            Ok(p) => Ok(p),
            Err(err) => Err(
                String::from(
"Failed to open port
    ensure port is COM[x] on windows or /dev/tty[x] on linux, corresponding to the linked bluetooth module
    port IO Error: " ) + &err.to_string())
        }
}

fn request_data_from_device(port: &mut Box<dyn serialport::SerialPort>) -> Result<(), String>{
    for _ in 0..2 {
        port.write(b"get\n\r").expect("Write failed!");

        let mut check_char : [u8; 1] = [0];
        port.read(check_char.as_mut_slice()).expect("    failed to read from port");
        match check_char[0] as char {
            'd' => println!("    negative response from device, retrying..."),
            'a' => { println!("    positive response from device!"); break; },
            _   => return Err(String::from("    Unexpected response from device: ") + &(check_char[0] as char).to_string())
        };
    }
    Ok(())
}

pub fn get_readings(port_label: &str) -> Result<Vec<u8>, String> {
    const SENSOR_READ_DELAY : u64 = 60000;
    const MAX_SENSOR_READINGS : usize = 10000;

    println!("    attempting to connect to port [{}]...", port_label);

    let mut port = match open_port_with_device(port_label, SENSOR_READ_DELAY) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    println!("    connected!\n    requesting readings...");

    match request_data_from_device(&mut port) {
        Err(e) => return Err(e),
        _ => ()
    }

    println!("\n    waiting for next sensor reading to get data ({} ms max)", SENSOR_READ_DELAY);

    let mut raw_data : Vec<u8> = vec![0; MAX_SENSOR_READINGS * 24]; //data in TT.T,HH.H,SSSSSSSSS...\n\r
    let mut error : Option<Error> = None;
    match port.read(raw_data.as_mut_slice()) {
        Ok(size) => size,
        Err(err) => {
                error = Some(err);
                0
            }
        };

    match error {
        Some(err) => Err(err.to_string()),
        None => Ok(raw_data)
    }
}

pub fn save_data(data : Vec<u8>, file_path : &str) {
    println!("    saving data to file..");
    let path = Path::new(file_path);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(msg) => panic!("    couldn't open {}: {}", display, msg),
        Ok(file) => file,
    };

    match file.write_all(&data) {
        Err(msg) => panic!("    couldn't write to {}: {}", display, msg),
        Ok(_) => println!("    successfully wrote data"),
    }
}
