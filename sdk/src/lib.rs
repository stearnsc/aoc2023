use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Output;
pub use log::{trace, debug, info, warn, error};
pub use anyhow::Result;
pub use anyhow;
pub use dotenvy;
pub use lazy_static::lazy_static;
pub use winnow;

pub fn init() {
    dotenvy::dotenv().expect(".env file not found");
    pretty_env_logger::init();
}

pub fn lines(path: impl AsRef<Path>) -> Result<impl Iterator<Item=String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().map(|l| l.expect("cannot read line")))
}

pub trait Solution<'a> {
    type Input: 'a;

    type Output: Display;

    fn parse<'i: 'a>(input: impl Iterator<Item=&'i str>) -> Result<Self::Input>;

    fn part_1(input: &Self::Input) -> Result<Output>;

    fn part_2(input: &Self::Input) -> Result<Output>;
}

#[derive(Debug)]
pub struct ParseError(pub String);

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}
