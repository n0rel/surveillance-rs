use config::{Config, ConfigError};
use serde::{Serialize, Deserialize};


#[derive(Deserialize, Serialize, Debug)]
pub struct SourceConfiguration {
    pub name: String,
    pub source_uri: String
}


#[derive(Deserialize, Serialize, Debug)]
pub struct Configuration {
    pub sources: Vec<SourceConfiguration>
}


/// Parses the configuration file into the `Configuration` struct
pub fn parse_configuration(file_path: &str) -> Result<Configuration, ConfigError> {
    Ok(
        Config::builder()
                .add_source(config::File::with_name(file_path))
                .build()?
                .try_deserialize::<Configuration>()?
    )
}