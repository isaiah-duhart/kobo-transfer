use std::env;
use kobo_transfer::{self, Config, transfer};

fn main() {
    let Some(config) = Config::build(env::args()) else {
        kobo_transfer::help();
        std::process::exit(1);
    };
        
    let Err(errors) = transfer(config) else {
        println!("Trasnfer completed!");
        return;
    };

    println!("{errors}");
}
