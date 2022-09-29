use core::fmt;
use std::{num::ParseIntError, str::FromStr};

use clap::Parser;

static EDITOR: &str = "hx";

#[derive(Clone, Copy, Debug)]
struct ElapsedTime {
    hours: i32,
    minutes: i32,
}

impl fmt::Display for ElapsedTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}+{}", self.hours, self.minutes)
    }
}

impl FromStr for ElapsedTime {
    type Err = ParseElapsedTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('+') {
            Some((hours, minutes)) => Ok(ElapsedTime {
                hours: hours.parse()?,
                minutes: minutes.parse()?,
            }),
            None => {
                let total_minutes: i32 = s.parse()?;
                let hours = total_minutes / 60;
                let minutes = total_minutes % 60;
                Ok(ElapsedTime { hours, minutes })
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum ParseElapsedTimeError {
    #[error(transparent)]
    Num(#[from] ParseIntError),
}

#[derive(Clone, Debug, Parser)]
struct Args {
    origin: String,

    /// waypoints
    /// 
    /// A collection of waypoints other than your point of origin. These should appear in order
    /// and the final waypoint should be your destination.
    #[arg(required(true))]
    waypoints: Vec<String>,

    /// elapsed time
    /// 
    /// Expressed in minutes or hours+minutes ("123" or "2+03")
    elapsed: ElapsedTime,
}

fn main() {
    run(&Args::parse());
}

fn run(args: &Args) {
    todo!()
}
