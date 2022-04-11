extern crate serialport;

use std::time::Duration;
use std::fs::File;
use std::path::Path;
use std::io::{Write, ErrorKind};
use std::thread::sleep;

const BYTES_IN_READING : usize = 7;

fn open_port_with_device(port: &str, timeout: u64) -> Result<Box<dyn serialport::SerialPort>, String> {
    let port = match serialport::new(port, 9600)
        .timeout(Duration::from_millis(timeout))
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

fn request_data_from_device(port: &mut Box<dyn serialport::SerialPort>) -> Result<(), String>{
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

pub fn get_readings(port_label: &str) -> Result<Vec<u8>, String> {
    const MAX_TIMEOUT : u64 = 10000;
    const SENSOR_READING_CHUNK : usize = 1000;
    const MAX_SENSOR_BUFFER : usize = 10000;

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

    let mut raw_data : Vec<u8> = vec![0; MAX_SENSOR_BUFFER * BYTES_IN_READING];
    let mut total_bytes_read = 0;
    for chunk in raw_data.chunks_mut(SENSOR_READING_CHUNK * BYTES_IN_READING) {
            let bytes_read = match port.read(chunk) {
                Ok(size) => size,
                Err(err) => {
                    if err.kind() != ErrorKind::TimedOut {
                        return Err(err.to_string());
                    }
                    println!("    Got IO timed out error, this could indicate an issue, or the device could have ran out of data to send on a chunk boundary");
                    0
                },
            };

            total_bytes_read += bytes_read;

            println!("    read {} bytes from device", bytes_read);

            if bytes_read != SENSOR_READING_CHUNK * BYTES_IN_READING {
                break;
            }
    }

    println!("    successfully read data from device!");

    let extra_data = raw_data.len() - total_bytes_read;
    for _ in 0..extra_data {
        if raw_data.pop().expect("nothing to pop from raw data!") != 0 {
            panic!("popped non-zero value from raw data buffer!");
        }
    }
    if raw_data.len() % BYTES_IN_READING != 0 {
        println!("    warning: Length of raw data isn't a multiple of bytes in reading! appending junk zeros to correct");
        while raw_data.len() % BYTES_IN_READING != 0 {
            println!("    zero appended");
            raw_data.push(0);
        }
    }
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
