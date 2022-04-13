extern crate serialport;

pub mod pico_interface;
pub mod csv;

const GET_COMMAND_FORMAT : &str = "get [port] [file] [optional args]...";
const GET_COMMAND_ARGS : &str =
"                --useprev
                    look for previous record in
                    specified file and use previous
                    timestamp as offset

                --useoffset [offset in seconds]
                    use specified offset for timestamps";

const DELAY_COMMAND_FORMAT : &str = "delay [port] [optional args]";
const DELAY_COMMAND_ARGS : &str =
"                --set [delay in seconds]
                    sets pico sensor delay to the given time in seconds";

pub fn help_command(_: Vec<String>) -> Result<(), String> {
    println!(
"    avaliable commands:

        {}

            opens connection to pico over [port] serial port
            and saves in csv format to new file at [file],
            or appends to existing file

            optional args:
{}

        {}
            opens connection to pico over [port] serial port
            and gets the current sensor reading delay in seconds

            optional args:
{}
", GET_COMMAND_FORMAT, GET_COMMAND_ARGS, DELAY_COMMAND_FORMAT, DELAY_COMMAND_ARGS);
    Ok(())
}


pub fn get_command(args: Vec<String>) -> Result<(), String> {
    let arg_error: String = String::from("        expected args:\n            ") + GET_COMMAND_FORMAT + "\n    pass 'help' for optional command list";

    let port = match args.get(0) {
        Some(arg) => {
            if arg == "help" || arg == "--help"  {
                println!("    {}, \n    optional args:\n{}", GET_COMMAND_FORMAT, GET_COMMAND_ARGS);
                return Err(String::from("get help displayed"));
            }
            arg
        },
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
                    "--useprev" => use_previous_time_offset = true,

                    "--useoffset" => {
                        i+=1;
                        extra_time_offset = match args.get(i) {
                            Some(num) => match num.parse::<u32>() {
                                Ok(s) => s,
                                Err(_) => return Err(String::from("arg Supplied with --useoffset is not a valid u32 number: \n")),
                            },
                            None => return Err(String::from("No offset supplied with --useoffset: \n    should be: --useoffset [offset]\n")),
                        }
                    }

                    _ => return Err(String::from("unknown optional arg supplied: ") + arg + "\n    pass 'help' for optional command list"),
                }
            },
            None => panic!("None recieved from optinal arg parse!"),
        }

        i+= 1;
    }

    let readings = pico_interface::get_readings(&port)?;

    csv::save_sensor_reading_bytes_as_csv(readings, &save_file, extra_time_offset, use_previous_time_offset)
}

pub fn delay_command(args: Vec<String>) -> Result<(), String> {
    let arg_error: String = String::from("        expected args:\n            ") + DELAY_COMMAND_FORMAT + "\n    pass 'help' for optional command list";

    let port = match args.get(0) {
        Some(arg) => {
            if arg == "help" || arg == "--help" {
                println!("    {}, \n    optional args:\n{}", DELAY_COMMAND_FORMAT, DELAY_COMMAND_ARGS);
                return Err(String::from("delay help displayed"));
            }
            arg
        },
        None => {
            return Err(String::from("Missing port argument: \n") + &arg_error + pico_interface::PORT_FORMAT_ERROR_MESSAGE);
        }
    };

    let mut set_time : u8 = 0;
    let mut i = 1;
    while i < args.len() {
        match args.get(i) {
            Some(arg) => {
                match arg.to_lowercase().as_str() {
                    "--set" => {
                        i+=1;
                        set_time = match args.get(i) {
                            Some(num) => match num.parse::<u8>() {
                                Ok(s) => s,
                                Err(_) => return Err(String::from("arg Supplied with --set is not a valid u8 number: \n")),
                            },
                            None => return Err(String::from("No offset supplied with --set: \n    should be: --set [delay in seconds]\n")),
                        }
                    }

                    _ => return Err(String::from("unknown optional arg supplied: ") + arg + "\n    pass 'help' for optional command list"),
                }
            },
            None => panic!("None recieved from optional arg parse!"),
        }

        i+= 1;
    }

    let got_time = pico_interface::pico_delay_command(port, set_time)?;

//0 means request current delay
    if set_time == 0 {
        println!("    pico's current delay is {} seconds", got_time);
    } else {
        println!("    successfully set pico's delay to {} seconds\n    note: this will only take effect after the next reading", got_time);
    }

    Ok(())
}
