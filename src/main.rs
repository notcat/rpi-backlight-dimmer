use std::{
    fmt::Error,
    fs::{self, File},
    io::{self, Read},
};
use thiserror::Error as ThisError;

const PATH_TO_BACKLIGHT: &str = "C:/Users/damon/OneDrive/Rust/rpi-backlight-dimmer/brightness";

#[derive(Debug, ThisError)]
pub enum RpiError {
    #[error("Unable to read file")]
    ReadError,
}

fn main() {
    let brightness = match read_backlight_from_file() {
        Ok(value) => value, // returning the value given from the result, and this sets the brightness variable
        Err(_) => RpiError::ReadError.to_string(),
    };

    dbg!(brightness);
    //println!("{}", data)

    //Err(RpiError::ReadError);
}

fn read_backlight_from_file() -> Result<String, io::Error> {
    let mut backlight_value = String::new();

    File::open(PATH_TO_BACKLIGHT)?.read_to_string(&mut backlight_value)?;

    Ok(backlight_value)
}
