mod cli;
mod config;

use cli::Cli;
use config::parse_configuration;


fn main() {

    let args = Cli::init();
    let configuration = parse_configuration(&args.config).unwrap();

    println!("{:?}", configuration)
}
