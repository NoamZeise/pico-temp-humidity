use std::env;

use pico_th_collector;

fn main() {
    let mut command : Option<String> = None;
    let mut command_args: Vec<String> = Vec::with_capacity(2);

    for (i, arg) in env::args().skip(1).enumerate() {
        match i {
            0 => command = Some(arg),
            _ => command_args.push(arg),
        }
    }

    if command == None {
        eprintln!("    No command specified!\n        help -> list avaliable commands")
    } else {
        let command_func : &dyn Fn(Vec<String>) -> Result<(), String>
            = match command.unwrap()
                           .to_lowercase()
                           .as_str() {
            "get" => &pico_th_collector::get_command,
            "delay" => &pico_th_collector::delay_command,
            "help" | "--help" => &pico_th_collector::help_command,
            _ => {
                eprintln!("    Unknown command!\n        help -> list avaliable commands\n    [command] help -> get specific command help");
                return;
            },
        };

        match command_func(command_args) {
            Err(e) => eprintln!("    Error :\n    {}", e),
            _ => (),
        }
    }

}
