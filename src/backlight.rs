use std::f32::consts::PI;
use std::io::{self, ErrorKind, Write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::{fs::File, io::Read};

use chrono::{self, Local, Timelike};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum RpiError {
    #[error("generic file error")]
    FileError,
    //FileError(#[from] io::Error),
    #[error("missing permissions to the file!")]
    MissingPermissions,
    #[error("file not found!")]
    FileMissing,
    // See line 39
    // #[error("file was busy")]
    // FileBusy,
    #[error("error while reading file contents")]
    ReadError,
    #[error("unable to parse file into a number value")]
    ParseError(ParseIntError),
}

pub struct RpiBacklight {
    path: PathBuf,
    timestates: TimeStates,
}

pub struct TimeStates {
    pub(crate) sunrise: f32,
    pub(crate) noon: f32,
    pub(crate) dusk: f32,
}

impl RpiBacklight {
    pub fn new(path: PathBuf, timestates: TimeStates) -> Self {
        Self { path, timestates }
    }

    pub fn read_backlight_from_file(&mut self) -> Result<u8, RpiError> {
        let mut backlight_string = String::new();

        let _file = File::open(&self.path); // .read_to_string(&mut backlight_value)

        let _file: Result<_, RpiError> = match _file {
            Ok(mut handle) => match handle.read_to_string(&mut backlight_string) {
                Ok(_) => Ok(()),
                Err(_) => return Err(RpiError::ReadError),
            },
            Err(error) => Err(Self::map_error(error)),
        };

        Self::trim_newline(&mut backlight_string);

        let result = backlight_string
            .parse::<u8>()
            .map_err(RpiError::ParseError)?;

        Ok(result)
    }

    pub fn change_backlight_in_file(&mut self, value: u8) -> Result<(), RpiError> {
        // let backlight_value = match self.read_backlight_from_file() {
        //     Ok(value) => value,
        //     Err(err) => return Err(err),
        // };

        let _file = File::options().write(true).truncate(true).open(&self.path); // .read_to_string(&mut backlight_value)

        match _file {
            Ok(mut handle) => match handle.write_all(value.to_string().as_bytes()) {
                Ok(_) => Ok(()),
                Err(error) => Err(Self::map_error(error)),
            },
            Err(error) => Err(Self::map_error(error)),
        }

        // lerp (current value, future value, time)
    }

    pub fn change_backlight_off_dayminute(&mut self) -> u8 {
        // chrono
        // time of day in MINUTES
        // convert minutes to 0-1 value
        // pipe daytimevalue into y = -0.5cos(x)+0.5 (remove negative for dusk curve)
        // x being the time

        let minutes_zero_to_one =
            ((Local::now().hour() * 60) + Local::now().minute()) as f32 / 1440.0;
        // let minutes_zero_to_one = Local::now().second() as f32 / 60.0;

        let mut calculated_brightness: f32 = 0.0;

        // Beginning rising curve
        if minutes_zero_to_one > self.timestates.sunrise
            && minutes_zero_to_one < self.timestates.noon
        {
            calculated_brightness = -0.5 * f32::cos(PI * minutes_zero_to_one * 3.0) + 0.5;
            // Pi * minutes_zero_to_one * 3 because there is 3 different time states.
        }

        // Constant High
        if minutes_zero_to_one > self.timestates.noon && minutes_zero_to_one < self.timestates.dusk
        {
            calculated_brightness = 1.0;
        }

        // falling rising curve
        if minutes_zero_to_one > self.timestates.dusk {
            calculated_brightness = 0.5 * f32::cos(PI * minutes_zero_to_one * 3.0) + 0.5;
        }

        (calculated_brightness * 245.0 + 10.0) as u8
    }

    fn trim_newline(string: &mut String) {
        if string.ends_with('\n') {
            string.pop();
            if string.ends_with('\r') {
                string.pop();
            }
        }
    }

    fn map_error(error: io::Error) -> RpiError {
        match error.kind() {
            ErrorKind::NotFound => RpiError::FileMissing,
            ErrorKind::PermissionDenied => RpiError::MissingPermissions,
            // See line 39
            //ErrorKind::ResourceBusy => return Err(RpiError::FileBusy),
            _ => RpiError::FileError,
        }
    }
}
