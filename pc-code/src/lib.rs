extern crate serialport;

use std::time::Duration;
use std::fs::File;
use std::path::Path;
use std::io::Write;

const BYTES_IN_READING : usize = 7;

fn open_port_with_device(port: &str, timeout: u64) -> Result<Box<dyn serialport::SerialPort>, String> {
    match serialport::new(port, 9600)
        .timeout(Duration::from_millis(timeout))
        .open() {
            Ok(p) => Ok(p),
            Err(err) => Err(
                String::from(
"    Failed to open port
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
            'e' => return Err(String::from("    Device Has nothing in buffer to send!")),
            _   => return Err(String::from("    Unexpected response from device: ") + &(check_char[0] as char).to_string()),
        };
    }
    Ok(())
}

pub fn get_readings(port_label: &str) -> Result<Vec<u8>, String> {
    const MAX_TIMEOUT : u64 = 5000;
    const MAX_SENSOR_READINGS : usize = 10000;

    println!("    attempting to connect to port [{}]...", port_label);

    let mut port = match open_port_with_device(port_label, MAX_TIMEOUT) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    println!("    connected!\n    requesting readings...");

    match request_data_from_device(&mut port) {
        Err(e) => return Err(e),
        _ => ()
    }

    println!("\n    waiting for data transmission completion ({} ms timeout)", MAX_TIMEOUT);

//data is 7 bytes per reading -> HmtyHmty TempTemp TimeTimeTime
//Hmty and Temp are 2 byte unsinged int, theres 1 byte representing left of decimal, 1 byte right of decimal
//Time is 3 byte unsinged int -> second byte is >> 8, third is  >> 16

    let mut raw_data : Vec<u8> = vec![0; MAX_SENSOR_READINGS * BYTES_IN_READING];
    let bytes_read = match port.read(raw_data.as_mut_slice()) {
        Ok(size) => size,
        Err(err) => return Err(err.to_string())
        };

    println!("    successfully read data from device!");

    let extra_data = raw_data.len() - bytes_read;
    for _ in 0..extra_data {
        if raw_data.pop().expect("nothing to pop from raw data!") != 0 {
            panic!("popped non-zero value from raw data buffer!");
        }
    }
    assert!(raw_data.len() % BYTES_IN_READING == 0, "Length of raw data isn't a multiple of bytes in reading!");

    Ok(raw_data)
}

fn to_csv_text(data: Vec<u8>) -> String {
    let mut csv_text = String::with_capacity(data.len() * BYTES_IN_READING * 3);
    csv_text.push_str("Humidity,Temperature,Time,\n");
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
    let csv_text = to_csv_text(data);

    println!("    saving data to file..");
    let path = Path::new(file_path);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(msg) => panic!("    couldn't open {}: {}", display, msg),
        Ok(file) => file,
    };

    match file.write_all(&csv_text.as_bytes()) {
        Err(msg) => panic!("    couldn't write to {}: {}", display, msg),
        Ok(_) => println!("    successfully wrote data"),
    }
}
