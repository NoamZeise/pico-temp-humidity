extern crate serialport;

use std::time::Duration;
use std::io::{Write, Error, ErrorKind, stdout};
use std::thread::sleep;


pub const BYTES_IN_READING : usize = 7;
pub const PORT_FORMAT_ERROR_MESSAGE : &str = "
    Ensure port is:
        COM[x] on windows
        /dev/tty[name] for usb on linux
        /dev/rfcomm[x] for bluetooth on linux
    corresponding to the paired bluetooth module or usb port\n";
const MAX_TIMEOUT : u64 = 500;
const MAX_SENSOR_BUFFER : usize = 10000;

fn open_port_with_device(port: &str) -> Result<Box<dyn serialport::SerialPort>, String> {
    let port = match serialport::new(port, 9600)
        .timeout(Duration::from_millis(MAX_TIMEOUT))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open() {
            Ok(p) => p,
            Err(err) => return Err( String::from("Failed to open port: ") + &err.to_string() +  "\n" +
                PORT_FORMAT_ERROR_MESSAGE),
    };

    //allow buffer to fill with any data from device, then clear before requesting
    sleep(Duration::from_millis(1000));
    port.clear(serialport::ClearBuffer::All).expect("Failed to clear buffers");

    Ok(port)
}

fn get_single_byte(port: &mut Box<dyn serialport::SerialPort>) -> Result<u8, String> {
    let mut check_char : [u8; 1] = [0];
    match port.read(check_char.as_mut_slice()) {
        Ok(_) => Ok(check_char[0]),
        Err(e) => return Err(String::from("Failed to read from port, Error: ") + &e.to_string()),
    }
}

fn send_command_to_device(port: &mut Box<dyn serialport::SerialPort>, command: &str) -> Result<(), String> {
    let mut got_response = false;
    for _ in 0..2 {
        port.write(command.as_bytes()).expect("Write failed!");

        let check_char = get_single_byte(port)?;
        match check_char {
            1 => {
                got_response = true;
                println!("    Positive response from device!");
                break;
            },
            2 => return Err(String::from("    Device Has nothing in buffer to send!")),
            3 => println!("    Negative response from device, retrying..."),
            _   => return Err(String::from("    Unexpected response from device: ") + &(check_char as char).to_string()),
        };
    }
    match got_response {
        false => Err(String::from("    No positive response from device")),
        true  => Ok(()),
    }
}

fn handle_reading_data_result(result : Result<usize, Error>) -> Result<usize, String>{
    match result {
        Ok(size) => Ok(size),
        Err(err) => {
            if err.kind() != ErrorKind::TimedOut {
                return Err(err.to_string());
            }
            println!("\n    Got IO timed out error, this could indicate an issue, or the device could have ran out of data to send on a chunk boundary");
            Ok(0)
        },
    }
}

fn check_end_transmission_and_sync(port: &mut Box<dyn serialport::SerialPort>) -> Result<bool, String> {
    let mut sync_char : [u8; 1] = [0];
    loop {
        let bytes_read = match handle_reading_data_result(port.read(sync_char.as_mut_slice())) {
            Ok(size) => size,
            Err(e) => return Err(e),
        };

        if bytes_read == 1 {
            match sync_char[0] {
                255 => return Ok(false), //continue
                254 => return Ok(true), //end transmission
                _   => println!("\n    Warning: incorrect sync char, reading until next sync"),
            }
        } else {
            println!("    Warning: no sync char read, continuing to save");
            return Ok(true);
        }
    }
}

fn collect_sensor_reading_bytes(port :  &mut Box<dyn serialport::SerialPort>) -> Result<Vec<u8>, String> {
    let mut raw_data : Vec<u8> = vec![0; MAX_SENSOR_BUFFER * BYTES_IN_READING];
    let mut total_bytes_read = 0;

    //data is 7 bytes per reading -> HmtyHmty TempTemp TimeTimeTime
    //Hmty and Temp are 2 byte unsinged int, theres 1 byte representing left of decimal, 1 byte right of decimal
    //Time is 3 byte unsinged int -> second byte is >> 8, third is  >> 16
    for chunk in raw_data.chunks_mut(BYTES_IN_READING) {
        let bytes_read = match handle_reading_data_result(port.read(chunk)) {
            Ok(size) => size,
            Err(e) => return Err(e),
        };

        for i in bytes_read..chunk.len() {
            println!("\n    Warning: incorrect chunk length! appending zeros");
            chunk[i] = 0;
        }

        total_bytes_read += bytes_read;

        if total_bytes_read % (BYTES_IN_READING * 100) == 0 {
            print!("    {} bytes read from device\r", total_bytes_read);
            match stdout().flush() {
                Ok(_) => (),
                Err(e) => println!("    error flushing stdout: {} ", e.to_string()),
            }
        }

        match check_end_transmission_and_sync(port) {
            Ok(end) => if end { break },
            Err(e) => return Err(e),
        }
    }
    raw_data.resize(total_bytes_read, 0);
    Ok(raw_data)
}

pub fn get_readings(port_label: &str) -> Result<Vec<u8>, String> {
    println!("    Attempting to connect to port [{}]...", port_label);
    let mut port = open_port_with_device(port_label)?;
    println!("    Connected!\n    Requesting readings...");
    send_command_to_device(&mut port, "get\r\n")?;
    println!("\n    Waiting for data transmission completion ({} ms timeout)", MAX_TIMEOUT);

    let mut raw_data : Vec<u8> = collect_sensor_reading_bytes(&mut port)?;

    println!("    {} bytes read from device", raw_data.len());
    println!("    Data read successfully");

    if raw_data.len() % BYTES_IN_READING != 0 {
        println!("    Warning: Length of raw data isn't a multiple of bytes in reading! appending junk zeros to correct");
        while raw_data.len() % BYTES_IN_READING != 0 {
            println!("    Zero appended");
            raw_data.push(0);
        }
    }

    Ok(raw_data)
}

pub fn pico_delay_command(port_label: &str, delay: u8) -> Result<u8, String> {
    println!("    Attempting to connect to port [{}]...", port_label);
    let mut port = open_port_with_device(port_label)?;
    println!("    Connected! sending command to device...");
    send_command_to_device(&mut port, "delay\r\n")?;
    println!("    Sending delay request");
    port.write(&[delay]).expect("Write failed!");

    let device_delay = get_single_byte(&mut port)?;

    if delay != 0 && device_delay != delay {
        return Err(String::from("Device delay doesn't match requested delay, device has delay of ") + &device_delay.to_string() + " seconds");
    }

    Ok(device_delay)
}
