use serde::Deserialize;
use std::env;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    // Networking settings
    pub port: u16,

    // Miner settings
    pub max_blocks: u64,
    pub max_nonce: u64,
    pub difficulty: u32,
    pub tx_waiting_ms: u64,
}

impl Config {
    pub fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
        // Open the file in read-only mode with buffer.
        let file = File::open(env::current_dir().unwrap().join(path))?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `Config`.
        let config = serde_json::from_reader(reader)?;

        // Return the `Config`.
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn successful_json_read() {
        let json_path = env::current_dir().unwrap().join("config.json");

        let config = Config::read_config_from_file(json_path.into_boxed_path()).unwrap();

        assert!(config.difficulty > 0);
    }

    #[test]
    #[should_panic]
    fn wrong_json_name() {
        let json_path = env::current_dir().unwrap().join("WRONG_config.json");

        let u = Config::read_config_from_file(json_path.into_boxed_path()).unwrap();
    }
}
