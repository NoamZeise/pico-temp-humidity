
use std::fs::{ File, OpenOptions };
use std::path::Path;
use std::env;
use std::io::{ Read, Write, Seek, SeekFrom };

use crate::pico_interface::BYTES_IN_READING;

fn hour_format_to_seconds(val: u32, index: u32) -> u32 {
    let multiples : [u32; 4]= [24, 60, 60, 1];
    let mut sum = val;
    assert!(index < 4, "hour format to seconds got an index above 4");
    let mut i: i32 = 3;
    while i >= index as i32 {
        sum *= multiples[i as usize];
        i -= 1;
    }
    sum
}

fn seconds_format_to_hour(val: u32) -> String {
    let multiples : [u32; 4] = [24, 60, 60, 1];
    let mut seconds_left = val;
    let mut formatted_time = String::with_capacity(11);
    for i in 0..4 {
        let mut corrected_time = seconds_left;
        let mut j = 4;
        while j > i {
            corrected_time /= multiples[j-1];
            j -= 1;
        }
        seconds_left -= hour_format_to_seconds(corrected_time, i as u32);

        let mut val = corrected_time.to_string();
        if val.len() == 1 {
            val.insert(0, '0');
        }
        formatted_time.push_str(&val);
        if i != 3 {
            formatted_time.push('-');
        }
    }
    formatted_time
}


fn to_csv_text(data: Vec<u8>, heading: bool, time_offset: u32, use_hour_format: bool) -> String {
    let mut csv_text = String::with_capacity(data.len() * BYTES_IN_READING * 4);
    if heading {
        csv_text.push_str("Time,Humidity,Temperature,\n");
    }
    for i in 0..data.len()/BYTES_IN_READING {
        //Time
        let time =   (data[(i*BYTES_IN_READING) + 4] as u32)        +
                    ((data[(i*BYTES_IN_READING) + 5] as u32) << 8)  +
                    ((data[(i*BYTES_IN_READING) + 6] as u32) << 16) +
                    time_offset;

        let time : String = match use_hour_format {
            true => seconds_format_to_hour(time),
            false => time.to_string()
        };

        csv_text.push_str(&time);
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

fn parse_string_to_u32(time: &str, failed_msg: &str) -> Result<u32, String>{
    match time.parse::<u32>() {
        Ok(s) => Ok(s),
        Err(_) => Err(String::from("    failed to convert prev time \"") + time +  "\" to u32 number\n" + failed_msg),
    }
}

fn parse_prev_time(time: &str, as_24: bool, failed_msg: &str) -> Result<u32, String> {
    if as_24 {
        let mut time = time.split("-");
        let mut offset : u32 = 0;
        for i in 0..4 {
            offset += hour_format_to_seconds(parse_string_to_u32(time.next().unwrap_or_else(|| { "Missing Time Record" }), failed_msg)?, i);
        }
        Ok(offset)
    } else {
        parse_string_to_u32(time, failed_msg)
    }
}

fn get_previous_time(file: &mut File, offset_with_prev_time: bool, prexisting_file: bool, as_24: bool) -> u32 {
    const PREVIOUS_OFFSET_FAILED_MSG : &str = "    Warning: Asked for offset with previous time, but no previous time could be found\n";
    if offset_with_prev_time {
        if prexisting_file {
            const LARGEST_POSSIBLE_LINE : usize = 26; //Time,Humidity,Temperature\n
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
                        Some(time) => {
                            match parse_prev_time(time, as_24, PREVIOUS_OFFSET_FAILED_MSG) {
                                Ok(num) => num,
                                Err(e) => {
                                     match parse_prev_time(time, !as_24, PREVIOUS_OFFSET_FAILED_MSG) {
                                        Ok(num) => {
                                            println!("    Warning: --astime was {}, but previosu record was in other format\n    converting to specified format", as_24);
                                            num
                                        },
                                        Err(_) => {
                                            println!("    Warning: failed to parse prev record, using 0 as offset\n    {}", e.to_string());
                                            0
                                        }
                                     }
                                 },
                            }
                        },
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

pub const SAVE_USE_PREVIOUS_TIME : u8 = 0b00000001;
pub const SAVE_USE_24_HOUR_FORMAT : u8 = 0b00000010;

pub fn save_sensor_reading_bytes_as_csv(data : Vec<u8>, file_path : &str, time_offset : u32, optional_args: u8) -> Result<(), String> {
    let offset_with_prev_time : bool = (optional_args & SAVE_USE_PREVIOUS_TIME) != 0;
    let use_hour_format : bool = (optional_args & SAVE_USE_24_HOUR_FORMAT) != 0;

    println!("\n    Saving data to file...");
    let path = Path::new(file_path);
    let mut prexisting_file = path.exists();

    let mut file = match OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .append(true)
                .open(path) {
        Err(msg) => {
            let mut file_path = env::temp_dir();
            file_path.push("picoTempHumidityReadings.csv");
            println!("    Couldn't open file, Error: {}\n\n    Attempting to save to temp directory as {}", msg.to_string(), file_path.display());
            prexisting_file = false;
            match OpenOptions::new()
                        .create(true)
                        .read(true)
                        .write(true)
                        .append(true)
                        .open(file_path) {
                            Ok(f) => {
                                println!("    Successfully saved to temp dir after failing to save to supplied file path");
                                f
                            },
                            Err(e) => return Err(String::from("    Couldn't create temp file after failing to save with original path file Error: ") + &e.to_string()),
                        }
        },
        Ok(file) => file,
    };

    let prev_time : u32 = get_previous_time(&mut file, offset_with_prev_time, prexisting_file, use_hour_format);

    let csv_text = to_csv_text(data, !prexisting_file, prev_time + time_offset, use_hour_format);

    match file.write_all(&csv_text.as_bytes()) {
        Err(msg) => return Err(String::from("    Couldn't write to file, Error: ") + &msg.to_string()),
        Ok(_) => println!("    Successfully wrote data to {}", file_path),
    }

    Ok(())
}
