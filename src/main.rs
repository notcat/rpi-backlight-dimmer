pub mod backlight;

use std::time::Duration;
use std::{error::Error, path::PathBuf};

use crate::backlight::{RpiBacklight, RpiError};

use backlight::TimeStates;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short, long)]
    pub path: PathBuf,
    #[structopt(short, long, default_value = "0.0")]
    pub sunrise: f32,
    #[structopt(short, long, default_value = "0.333")]
    pub noon: f32,
    #[structopt(short, long, default_value = "0.666")]
    pub dusk: f32,
}

impl Opt {
    pub fn timestates(&self) -> TimeStates {
        TimeStates {
            sunrise: self.sunrise,
            noon: self.noon,
            dusk: self.dusk,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let mut backlight = RpiBacklight::new(opt.path.clone(), opt.timestates());

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

        let new_value = backlight.change_backlight_off_dayminute();

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
                    dbg!(
                        "retrying in 3 seconds, (file could just be busy, or missing permissions)"
                    );

                    std::thread::sleep(Duration::from_secs(3));

                    continue;
                }
            }
        }

        println!("old value: {} -> new value: {}", brightness, new_value);

        std::thread::sleep(Duration::from_secs(1));
    }
}
