use clap::Parser;

const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
    pub config: String,
}

impl Cli {
    /// Initialization function for the Cli object.
    /// Useful for decoupling any other modules using this
    /// object from the `clap` crate
    pub fn init() -> Self {
        Cli::parse()
    }
}
