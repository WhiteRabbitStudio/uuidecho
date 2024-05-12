use std::fmt::Display;
use std::fs::File;
use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;

use clap::{arg, command, value_parser};
use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use uuid::Uuid;
use chrono::prelude::*;

#[derive(Debug)]
enum UUIDError {
    IoError(std::io::Error),
    InvalidInput(String),
}

impl From<std::io::Error> for UUIDError {
    fn from(err: std::io::Error) -> Self {
        UUIDError::IoError(err)
    }
}

impl Display for UUIDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UUIDError::IoError(err) => write!(f, "IO Error: {}", err),
            UUIDError::InvalidInput(err) => write!(f, "Invalid Input: {}", err),
        }
    }
}

fn main() -> Result<(), UUIDError> {
    let matches = command!("uuidecho")
        .arg(
            arg!(
                --file <FILE>
            ).required(false)
                .value_parser(value_parser!(PathBuf))
        ).
        arg(
            arg!(
                --input <VALUE>
            ).required(false)
        )
        .get_matches();

    let uuid  = if let Some(file) = matches.get_one::<PathBuf>("file") {
        let mut fh = File::open(file)?;
        read_from_buff(&mut fh)?
    } else if let Some(input) = matches.get_one::<String>("input") {
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, input.as_bytes())
    } else if !std::io::stdin().is_terminal() {
        let mut sth = std::io::stdin();
        read_from_buff(&mut sth)?
    } else {
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, Utc::now().to_string().as_bytes())
    };

    let mut std_out_handle = std::io::stdout().lock();
    write!(std_out_handle, "{}\n", uuid.hyphenated())?;

    Ok(())
}

fn read_from_buff<T: Read>(fh: &mut T) -> Result<Uuid, std::io::Error> {
    let mut buff = [0; 1024];
    let mut hasher = Context::new(&SHA256);

    loop {
        let count = fh.read(&mut buff)?;
        if count == 0 {
            break;
        }
        hasher.update(&buff[..count]);
        fh.read(&mut buff)?;
    }
    let digest = hasher.finish();

    Ok(uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, HEXUPPER.encode(digest.as_ref()).as_bytes()))
}