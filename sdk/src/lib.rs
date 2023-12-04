use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;
pub use log::{trace, debug, info, warn, error};
pub use anyhow::Result;
pub use anyhow;
pub use dotenvy;
pub use lazy_static::lazy_static;

pub fn init() {
    dotenvy::dotenv().expect(".env file not found");
    pretty_env_logger::init();
}

pub fn lines(path: impl AsRef<Path>) -> Result<impl Iterator<Item=io::Result<String>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}
