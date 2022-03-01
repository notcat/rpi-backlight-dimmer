use std::f32::consts::PI;
use std::io::{self, ErrorKind, Write};
use std::num::ParseIntError;
use std::str::Bytes;
use std::time::Duration;
use std::{any, error, time};
use std::{error::Error, fs::File, io::Read};

use chrono::{self, Local, Timelike};
use thiserror::Error as ThisError;

// TODO: steal specners structopt stuff and use that to fill out the PATH_TO_BACKLIGHT instead
// so you could just call ./rpi-backlight-dimmer --path /sys/class/backlight/rpi-backlight/brightness

const PATH_TO_BACKLIGHT: &str = "C:/Users/damooo/OneDrive/Rust/rpi-backlight-dimmer/brightness";

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut backlight = RpiBacklight::new(PATH_TO_BACKLIGHT);

    match backlight.read_backlight_from_file() {
        Ok(value) => println!("{}", value),
        Err(error) => match error {
            // Soon will be available in a new version of rust (currently on nightly only https://github.com/rust-lang/rust/issues/86442)
            // RpiError::FileBusy => {
            //     dbg!("got minor error!: {}", error);
            //     dbg!("retrying in 3 seconds, (file could just be busy)");

            //     std::thread::sleep(Duration::from_secs(3));

            //     continue;
            // }
            RpiError::FileError | RpiError::FileMissing => {
                panic!("{}", error);
            }
            RpiError::ParseError(error) => {
                dbg!(error.clone());
                panic!("{}", error);
            }
            other => {
                dbg!("got minor error!", other);
                dbg!("retrying in 3 seconds, (file could just be busy)");

                std::thread::sleep(Duration::from_secs(3));
            }
        },
    }

    loop {
        let brightness = match backlight.read_backlight_from_file() {
            Ok(value) => value,
            Err(error) => match error {
                // Soon will be available in a new version of rust (currently on nightly only https://github.com/rust-lang/rust/issues/86442)
                // RpiError::FileBusy => {
                //     dbg!("got minor error!: {}", error);
                //     dbg!("retrying in 3 seconds, (file could just be busy)");

                //     std::thread::sleep(Duration::from_secs(3));

                //     continue;
                // }
                RpiError::FileError | RpiError::FileMissing => {
                    panic!("{}", error);
                }
                RpiError::ParseError(error) => {
                    dbg!(error.clone());
                    panic!("{}", error);
                }
                other => {
                    dbg!("got minor error!", other);
                    dbg!("retrying in 3 seconds, (file could just be busy)");

                    std::thread::sleep(Duration::from_secs(3));

                    continue;
                }
            },
        };

        let mut new_value = backlight.change_backlight_off_dayminute();

        if new_value == brightness {
            std::thread::sleep(Duration::from_secs(1));
            continue;
        }

        if let Err(error) = backlight.change_backlight_in_file(new_value) {
            match error {
                RpiError::FileError | RpiError::FileMissing => {
                    panic!("{}", error)
                }
                RpiError::ParseError(error) => {
                    panic!("{}", error)
                }
                other => {
                    dbg!("got minor error!: {}", other);
                    dbg!("retrying in 3 seconds, (file could just be busy)");

                    std::thread::sleep(Duration::from_secs(3));

                    continue;
                }
            }
        }

        println!("old value: {} -> new value: {}", brightness, new_value);

        std::thread::sleep(Duration::from_secs(1));
    }
}

pub struct RpiBacklight {
    path: &'static str,
}

pub struct TimeStates {
    sunrise: f32,
    daytime: f32,
    dusk: f32,
}

impl RpiBacklight {
    pub fn new(path: &'static str) -> Self {
        Self { path }
    }

    fn read_backlight_from_file(&mut self) -> Result<u8, RpiError> {
        let mut backlight_string = String::new();

        let _file = File::open(self.path); // .read_to_string(&mut backlight_value)

        let _file: Result<_, RpiError> = match _file {
            Ok(mut handle) => match handle.read_to_string(&mut backlight_string) {
                Ok(_) => Ok(()),
                Err(_) => return Err(RpiError::ReadError),
            },
            Err(error) => Err(Self::map_error(error)),
        };

        let result = backlight_string
            .parse::<u8>()
            .map_err(|error| RpiError::ParseError(error))?;

        Ok(result)
    }

    fn change_backlight_in_file(&mut self, value: u8) -> Result<(), RpiError> {
        // let backlight_value = match self.read_backlight_from_file() {
        //     Ok(value) => value,
        //     Err(err) => return Err(err),
        // };

        let _file = File::options().write(true).truncate(true).open(self.path); // .read_to_string(&mut backlight_value)

        match _file {
            Ok(mut handle) => match handle.write_all(value.to_string().as_bytes()) {
                Ok(_) => Ok(()),
                Err(error) => Err(Self::map_error(error)),
            },
            Err(error) => Err(Self::map_error(error)),
        }

        // lerp (current value, future value, time)
    }

    fn change_backlight_off_dayminute(&mut self) -> u8 {
        // chrono
        // time of day in MINUTES
        // convert minutes to 0-1 value
        // pipe daytimevalue into y = -0.5cos(x)+0.5 (remove negative for dusk curve)
        // x being the time

        let minutes_zero_to_one =
            ((Local::now().hour() * 60) + Local::now().minute()) as f32 / 1440.0;
        // let minutes_zero_to_one = Local::now().second() as f32 / 60.0;

        // TODO: make this a config file option
        let timestates = TimeStates {
            sunrise: 0.0,
            daytime: 0.3333333,
            dusk: 0.6666666,
        };

        let mut calculated_brightness: f32 = 0.0;

        // Beginning rising curve
        if minutes_zero_to_one > timestates.sunrise && minutes_zero_to_one < timestates.daytime {
            calculated_brightness = -0.5 * f32::cos(PI * minutes_zero_to_one * 3.0) + 0.5;
        }

        // Constant High
        if minutes_zero_to_one > timestates.daytime && minutes_zero_to_one < timestates.dusk {
            calculated_brightness = 1.0;
        }

        // falling rising curve
        if minutes_zero_to_one > timestates.dusk {
            calculated_brightness = 0.5 * f32::cos(PI * minutes_zero_to_one * 3.0) + 0.5;
        }

        return (calculated_brightness * 250.0 + 5.0) as u8;
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
