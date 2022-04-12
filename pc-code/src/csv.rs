
use std::fs::{ File, OpenOptions };
use std::path::Path;
use std::io::{ Read, Write, Seek, SeekFrom };

use crate::pico_interface::BYTES_IN_READING;


fn to_csv_text(data: Vec<u8>, heading: bool, time_offset: u32) -> String {
    let mut csv_text = String::with_capacity(data.len() * BYTES_IN_READING * 4);
    if heading {
        csv_text.push_str("Time,Humidity,Temperature,\n");
    }
    for i in 0..data.len()/BYTES_IN_READING {
        //Time
        csv_text.push_str(
                ((data[(i*BYTES_IN_READING) + 4] as u32)        +
                ((data[(i*BYTES_IN_READING) + 5] as u32) << 8)  +
                ((data[(i*BYTES_IN_READING) + 6] as u32) << 16) +
                time_offset
            ).to_string().as_str()
        );
        csv_text.push(',');

        //Humidity
        csv_text.push_str((
            (data[(i*BYTES_IN_READING) + 0] as f32) + (data[(i*BYTES_IN_READING) + 1] as f32)/10.0)
                .to_string().as_str()
        );
        csv_text.push(',');

        //Temperature
        csv_text.push_str((
            (data[(i*BYTES_IN_READING) + 2] as f32) + (data[(i*BYTES_IN_READING) + 3] as f32)/10.0)
                .to_string().as_str()
        );
        csv_text.push(',');

        csv_text.push('\n');
    }
    csv_text
}

fn get_previous_time(file: &mut File, offset_with_prev_time: bool, prexisting_file: bool) -> u32 {
    const PREVIOUS_OFFSET_FAILED_MSG : &str = "    Warning: Asked for offset with previous time, but no previous time could be found\n";
    if offset_with_prev_time {
        if prexisting_file {
            const LARGEST_POSSIBLE_LINE : usize = 18;
            match file.seek(SeekFrom::End(-(LARGEST_POSSIBLE_LINE as i64))) {
                Ok(_) => (),
                Err(_) => {
                    println!("    failed to go back {} characters in file\n{}", LARGEST_POSSIBLE_LINE, PREVIOUS_OFFSET_FAILED_MSG);
                    return 0;
                },
            }

            let mut read_buff: Vec<u8> = vec![0; LARGEST_POSSIBLE_LINE - 2]; //skip ,\n

            match file.read_exact(read_buff.as_mut_slice()) {
                Ok(_) => (),
                Err(_) => {
                    println!("    failed to read last {} characters\n{}", LARGEST_POSSIBLE_LINE, PREVIOUS_OFFSET_FAILED_MSG);
                    return 0;
                },
            }

            file.seek(SeekFrom::End(0)).unwrap();

            let read_buff = match String::from_utf8(read_buff) {
                Ok(string) => string,
                Err(_) => {
                    println!("    failed to convert last line to string, invalid utf8\n{}", PREVIOUS_OFFSET_FAILED_MSG);
                    return 0;
                },
            };

            match read_buff.split('\n').last() {
                Some(last_line) => {
                    match last_line.split(',').next() {
                        Some(time) => match time.parse::<u32>() {
                            Ok(s) => s,
                            Err(_) => {
                                println!("    failed to convert prev time \"{}\" to u32 number\n{}", time, PREVIOUS_OFFSET_FAILED_MSG);
                                0
                            },
                        }
                        None => {
                            println!("    failed to get first column of last line\n{}", PREVIOUS_OFFSET_FAILED_MSG);
                            0
                        },
                    }
                },
                None => {
                    println!("    failed to get last line\n{}", PREVIOUS_OFFSET_FAILED_MSG);
                    0
                },
            }
        } else {
            println!("    no previous file exists\n{}", PREVIOUS_OFFSET_FAILED_MSG);
            0
        }
    } else {
        0
    }
}

pub fn save_sensor_reading_bytes_as_csv(data : Vec<u8>, file_path : &str, time_offset : u32, offset_with_prev_time: bool) -> Result<(), String> {

    println!("\n    Saving data to file..");
    let path = Path::new(file_path);
    let prexisting_file = path.exists();

    let mut file = match OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .append(true)
                .open(path) {
        Err(msg) => return Err(String::from("    Couldn't open file, Error: ") + &msg.to_string()),
        Ok(file) => file,
    };

    let prev_time : u32 = get_previous_time(&mut file, offset_with_prev_time, prexisting_file);

    let csv_text = to_csv_text(data, !prexisting_file, prev_time + time_offset);

    match file.write_all(&csv_text.as_bytes()) {
        Err(msg) => return Err(String::from("    Couldn't write to file, Error: ") + &msg.to_string()),
        Ok(_) => println!("    Successfully wrote data to {}", file_path),
    }

    Ok(())
}
