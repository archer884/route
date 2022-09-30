use std::{
    borrow::Cow,
    env, fmt, fs, io, iter,
    num::ParseIntError,
    ops::Not,
    process::{self, Command},
    str::FromStr,
};

use chrono::Duration;
use clap::Parser;
use serde::Serialize;
use serde_with::{self, serde_as};

static EDITOR: &str = "hx";

#[derive(Debug, thiserror::Error)]
enum ParseElapsedTimeError {
    #[error(transparent)]
    Num(#[from] ParseIntError),
}

#[derive(Clone, Copy, Debug)]
struct ElapsedTime {
    hours: i32,
    minutes: i32,
}

impl ElapsedTime {
    fn into_duration(self) -> Duration {
        Duration::hours(self.hours as i64) + Duration::minutes(self.minutes as i64)
    }
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

    /// notes on the flight
    ///
    /// If this field is left empty, an editor window will open and the user may save a comment
    /// there.
    #[arg(short, long)]
    notes: Option<String>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize)]
struct WriteFlight<'a> {
    waypoints: Vec<&'a str>,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    elapsed: Duration,
    notes: Option<Cow<'a, str>>,
}

fn main() {
    if let Err(e) = run(&Args::parse()) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(args: &Args) -> io::Result<()> {
    let notes = args
        .notes
        .as_deref()
        .map(|message| Ok(Cow::Borrowed(message)))
        .unwrap_or_else(|| read_from_file().map(Cow::Owned))?;

    let flight = WriteFlight {
        waypoints: iter::once(&*args.origin)
            .chain(args.waypoints.iter().map(|wpt| wpt.as_ref()))
            .collect(),
        elapsed: args.elapsed.into_duration(),
        notes: notes.is_empty().not().then_some(notes),
    };

    // FIXME: As demonstrated below, the program can make a Flight model now. Unfortunately,
    // it has no way of logging them as yet, so... keep at it!

    let data = serde_json::to_string(&flight).unwrap();
    println!("{data}");

    Ok(())
}

fn read_from_file() -> io::Result<String> {
    static HELP_MESSAGE: &str = include_str!("../resource/help_message.txt");

    let path = env::temp_dir().join("EDIT_COMMENT");

    fs::write(&path, HELP_MESSAGE)?;
    Command::new(EDITOR).arg(&path).status()?;

    fs::read_to_string(&path).map(strip_comments)
}

fn strip_comments(notes: String) -> String {
    let mut buf = String::with_capacity(notes.len());

    for line in notes.lines() {
        if !line.starts_with('#') {
            buf.push_str(line);
            buf.push('\n');
        }
    }

    // If it sucks but it works, it... still sucks.
    if buf.ends_with('\n') {
        buf.truncate(buf.len() - 1);
    }

    buf
}
