extern crate serialport;

use std::time::Duration;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::{Write, Error, ErrorKind};
use std::thread::sleep;

const BYTES_IN_READING : usize = 7;
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
            Err(err) => return Err(
                String::from(
"    Failed to open port
    ensure port is COM[x] on windows or /dev/tty[x] on linux, corresponding to the linked bluetooth module
    port IO Error: " ) + &err.to_string()),
    };

    //allow buffer to fill with any data from device, then clear before requesting
    sleep(Duration::from_millis(1000));
    port.clear(serialport::ClearBuffer::All).expect("failed to clear buffers");

    Ok(port)
}

fn request_sensor_readings_from_device(port: &mut Box<dyn serialport::SerialPort>) -> Result<(), String>{
    let mut got_response = false;
    for _ in 0..2 {
        port.write(b"get\n\r").expect("Write failed!");

        let mut check_char : [u8; 1] = [0];
        port.read(check_char.as_mut_slice()).expect("    failed to read from port");
        match check_char[0] {
            1 => {
                got_response = true;
                println!("    positive response from device!");
                break;
            },
            2 => return Err(String::from("    Device Has nothing in buffer to send!")),
            3 => println!("    negative response from device, retrying..."),
            _   => return Err(String::from("    Unexpected response from device: ") + &(check_char[0] as char).to_string()),
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
                _   => println!("\n    incorrect sync char, reading until next sync"),
            }
        } else {
            panic!("no sync char!");
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

        if total_bytes_read % (BYTES_IN_READING * 250) == 0 {
            print!("    {} bytes read from device\r", total_bytes_read);
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

    println!("    attempting to connect to port [{}]...", port_label);

    let mut port = match open_port_with_device(port_label) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    println!("    connected!\n    requesting readings...");

    match request_sensor_readings_from_device(&mut port) {
        Err(e) => return Err(e),
        _ => ()
    }

    println!("\n    waiting for data transmission completion ({} ms timeout)", MAX_TIMEOUT);

    let mut raw_data : Vec<u8> = match collect_sensor_reading_bytes(&mut port) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    println!("\n    successfully read {} bytes of data from device!", raw_data.len());

    if raw_data.len() % BYTES_IN_READING != 0 {
        println!("    warning: Length of raw data isn't a multiple of bytes in reading! appending junk zeros to correct");
        while raw_data.len() % BYTES_IN_READING != 0 {
            println!("    zero appended");
            raw_data.push(0);
        }
    }

    Ok(raw_data)
}

fn to_csv_text(data: Vec<u8>, heading: bool) -> String {
    let mut csv_text = String::with_capacity(data.len() * BYTES_IN_READING * 4);
    if heading {
        csv_text.push_str("Humidity,Temperature,Time,\n");
    }
    for i in 0..data.len()/BYTES_IN_READING {
        csv_text.push_str((
            (data[(i*BYTES_IN_READING) + 0] as f32) + (data[(i*BYTES_IN_READING) + 1] as f32)/10.0)
                .to_string().as_str()
        );
        csv_text.push(',');
        csv_text.push_str((
            (data[(i*BYTES_IN_READING) + 2] as f32) + (data[(i*BYTES_IN_READING) + 3] as f32)/10.0)
                .to_string().as_str()
        );
        csv_text.push(',');
        csv_text.push_str(
                ((data[(i*BYTES_IN_READING) + 4] as u32)       +
                ((data[(i*BYTES_IN_READING) + 5] as u32) << 8) +
                ((data[(i*BYTES_IN_READING) + 6] as u32) << 16)
            ).to_string().as_str()
        );
        csv_text.push(',');
        csv_text.push('\n');
    }
    csv_text
}

pub fn save_data_as_csv(data : Vec<u8>, file_path : &str) {

    println!("\n    saving data to file..");
    let path = Path::new(file_path);
    let display = path.display();

    let csv_text = to_csv_text(data, !path.exists());

    let mut file = match OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path) {
        Err(msg) => panic!("    couldn't open {}: {}", display, msg),
        Ok(file) => file,
    };

    match file.write_all(&csv_text.as_bytes()) {
        Err(msg) => panic!("    couldn't write to {}: {}", display, msg),
        Ok(_) => println!("    successfully wrote data"),
    }
}
