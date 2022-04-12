extern crate serialport;

pub mod pico_interface;
pub mod csv;

const GET_COMMAND_FORMAT : &str = "get [port] [file] [optional args]...";

pub fn help_command(_: Vec<String>) -> Result<(), String> {
    println!(
"    avaliable commands:

        {}

            opens connection to pico over [port] serial port
            and saves in csv format to new file at [file],
            or appends to existing file

            optional args:
                -useprev
                    look for previous timestamp in
                    specified file and use as offset

                -useoffset [offset]
                    use specified offset for timestamps
", GET_COMMAND_FORMAT);
    Ok(())
}


pub fn get_command(args: Vec<String>) -> Result<(), String> {
    let arg_error: String = String::from("        expected args:\n            ") + GET_COMMAND_FORMAT;

    let port = match args.get(0) {
        Some(arg) => arg,
        None => {
            return Err(String::from("Missing port argument: \n") + &arg_error + pico_interface::PORT_FORMAT_ERROR_MESSAGE);
        }
    };

    let save_file = match args.get(1) {
        Some(arg) => arg,
        None => {
            return Err(String::from("Missing file argument: \n") + &arg_error);
        }
    };

    let mut use_previous_time_offset = false;
    let mut extra_time_offset : u32 = 0;

    let mut i = 2;
    while i < args.len() {
        match args.get(i) {
            Some(arg) => {
                match arg.to_lowercase().as_str() {
                    "-useprev" => use_previous_time_offset = true,

                    "-useoffset" => {
                        i+=1;
                        extra_time_offset = match args.get(i) {
                            Some(num) => match num.parse::<u32>() {
                                Ok(s) => s,
                                Err(_) => return Err(String::from("arg Supplied with -useoffset is not a valid u32 number: \n")),
                            },
                            None => return Err(String::from("No offset supplied with -useoffset: \n    should be: -useoffset [offset]\n")),
                        }
                    }

                    _ => return Err(String::from("unknown optional arg supplied: \n   use 'help' for full command list"))
                }
            },
            None => panic!("None recieved from optinal arg parse!"),
        }

        i+= 1;
    }

    let readings = match pico_interface::get_readings(&port) {
        Ok(k) => k,
        Err(e) => return Err(e),
    };

    csv::save_sensor_reading_bytes_as_csv(readings, &save_file, extra_time_offset, use_previous_time_offset)
}
