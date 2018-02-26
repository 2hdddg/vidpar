use std::result;
use std::error::Error;
use std::fmt;

pub mod bitreader;
pub mod nalunit;
pub mod sps;
pub mod pps;

type Result<T> = result::Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserUnit {
    Nal(),
    Pps(),
    Sps(),
}

#[derive(Debug)]
pub struct ParserUnitError {
    unit: ParserUnit,
    description: String,
}

#[derive(Debug)]
pub struct BitReaderError {
    description: String,
}

#[derive(Debug)]
pub enum ParserError {
    BitReaderError(BitReaderError),
    BitReaderEndOfStream(),
    InvalidStream(ParserUnitError),
    NotImplemented(ParserUnitError),
}

impl Error for BitReaderError {
    fn description(&self) -> &str {
        &self.description
    }
}

impl fmt::Display for BitReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl Error for ParserUnitError {
    fn description(&self) -> &str {
        &self.description
    }
}

impl fmt::Display for ParserUnitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
