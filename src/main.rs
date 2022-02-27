use std::io::ErrorKind;
use std::time::Duration;
use std::{error::Error, fs::File, io::Read};

use thiserror::Error as ThisError;

// steal specners structopt stuff and use that to fill out the PATH_TO_BACKLIGHT instead
// so you could just call ./rpi-backlight-dimmer --path /sys/class/backlight/rpi-backlight/brightness

const PATH_TO_BACKLIGHT: &str = "C:/Users/damon/OneDrive/Rust/rpi-backlight-dimmer/brightness";

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
    ParseError,
}

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        let mut backlight = RpiBacklight::new(PATH_TO_BACKLIGHT);

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
                RpiError::FileError | RpiError::FileMissing | RpiError::ParseError => {
                    panic!("{}", error)
                }
                other => {
                    dbg!("got minor error!: {}", other);
                    dbg!("retrying in 3 seconds, (file could just be busy)");

                    std::thread::sleep(Duration::from_secs(3));

                    continue;
                }
            },
        };

        dbg!(brightness);
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub struct RpiBacklight {
    path: &'static str,
}

impl RpiBacklight {
    pub fn new(path: &'static str) -> Self {
        Self { path }
    }

    fn read_backlight_from_file(&mut self) -> Result<u8, RpiError> {
        let mut backlight_value = String::new();

        let _file = File::open(self.path); // .read_to_string(&mut backlight_value)

        let _file: Result<_, RpiError> = match _file {
            Ok(mut handle) => match handle.read_to_string(&mut backlight_value) {
                Ok(_) => Ok(()),
                Err(_) => return Err(RpiError::ReadError),
            },
            Err(error) => match error.kind() {
                ErrorKind::NotFound => return Err(RpiError::FileMissing),
                ErrorKind::PermissionDenied => return Err(RpiError::MissingPermissions),
                // See line 39
                //ErrorKind::ResourceBusy => return Err(RpiError::FileBusy),
                _ => return Err(RpiError::FileError),
            },
        };

        let result = backlight_value
            .parse::<u8>()
            .map_err(|_| RpiError::ParseError)?;

        Ok(result)
    }
}
