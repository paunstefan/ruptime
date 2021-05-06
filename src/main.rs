//! Rust implementation of uptime
use chrono::prelude::*;
use chrono::Duration;
use std::{env, error::Error, fmt, fs, io::Read};

const UTMP_SIZE: usize = 384;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        usage();
        return Err(Box::new(ArgsError));
    }
    let arg = args.get(1);

    let local: DateTime<Local> = Local::now();
    let uptime = get_uptime(fs::read_to_string("/proc/uptime")?)?;

    match arg {
        None => {
            let current_time_str = local.format("%H:%M:%S").to_string();
            let uptime_str = build_uptime_string(&uptime, UptimeFormat::Normal);
            let loadavg_str = get_loadavg(fs::read_to_string("/proc/loadavg")?);

            let mut buf: Vec<u8> = Vec::new();
            let mut f = fs::File::open("/var/run/utmp")?;
            f.read_to_end(&mut buf)?;
            let no_users = get_no_users(&buf);

            println!(
                " {} up {}, {}, load average: {}",
                current_time_str, uptime_str, no_users, loadavg_str
            );
        }
        Some(option) => match option.as_str() {
            "-p" | "--pretty" => {
                let uptime_str = build_uptime_string(&uptime, UptimeFormat::Pretty);
                println!("up {}", uptime_str);
            }
            "-h" | "--help" => usage(),
            "-s" | "--since" => {
                let since_datetime = local - uptime;
                println!("{}", since_datetime.format("%Y-%m-%d %H:%M:%S").to_string())
            }
            "-V" | "--version" => println!("ruptime 0.1.0"),
            _ => {
                usage();
                return Err(Box::new(ArgsError));
            }
        },
    }

    Ok(())
}

#[derive(Debug)]
struct ArgsError;

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Arguments error")
    }
}
impl Error for ArgsError {}

#[derive(Debug, PartialEq)]
enum UptimeFormat {
    Normal,
    Pretty,
}

/// Builds a formatted string with the uptime
/// Optionally it can be pretty
fn build_uptime_string(uptime: &Duration, kind: UptimeFormat) -> String {
    let mut result = String::new();
    let days = uptime.num_days();
    let hours = uptime.num_hours() - Duration::days(days).num_hours();
    let minutes = uptime.num_minutes()
        - Duration::hours(hours).num_minutes()
        - Duration::days(days).num_minutes();

    if days > 0 {
        result.push_str(&format!("{} ", days));
        result.push_str(if days == 1 { "day, " } else { "days, " });
    }

    if kind == UptimeFormat::Normal {
        let fmt_string = if hours == 0 {
            format!("{} min", minutes)
        } else {
            format!("{}:{:0>2}", hours, minutes)
        };
        result.push_str(&fmt_string);
    } else {
        let fmt_string = if hours == 0 {
            format!("{} minutes", minutes)
        } else {
            format!("{} hours, {} minutes", hours, minutes)
        };
        result.push_str(&fmt_string);
    }

    result
}

/// Get a Duration object with the uptime in seconds
fn get_uptime(read_data: String) -> Result<Duration, Box<dyn Error>> {
    let time_str: String = read_data.split_whitespace().take(1).collect();

    // Conversion can fail, but Duration works with integers
    let seconds = time_str.parse::<f64>()? as i64;

    Ok(Duration::seconds(seconds))
}

/// Reads /proc/loadavg and formats the result
fn get_loadavg(read_data: String) -> String {
    let load: Vec<String> = read_data
        .split_whitespace()
        .take(3)
        .map(|x| x.to_string())
        .collect();

    let load_str = format!("{}, {}, {}", load[0], load[1], load[2]);

    load_str
}

/// Print usage information
fn usage() {
    println!("\nUsage:");
    println!(" ruptime [option]\n");

    println!("Options:");
    println!(" -p, --pretty   show uptime in pretty format");
    println!(" -h, --help     display this help and exit");
    println!(" -s, --since    system up since");
    println!(" -V, --version  output version information and exit\n");
}

/// Get the number of logged users by reading the utmp file
fn get_no_users(buf: &[u8]) -> String {
    let mut count = 0;

    for i in 0..(buf.len() / UTMP_SIZE) {
        // At the start of each structure there is the type field
        // 7 is the USER_PROCESS type
        if buf[i * UTMP_SIZE] == 7 {
            count += 1;
        }
    }

    if count > 1 {
        format!("{} users", count)
    } else {
        format!("{} user", count)
    }
}
