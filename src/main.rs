use std::env;
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // Prints input text word-by-word with a configurable delay.
    // Feed lyrics via stdin: `cat your_lyrics.txt | cargo run -- --delay-ms 150`

    let mut delay_ms: u64 = 200;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--delay-ms" => {
                let val = args.next().unwrap_or_else(|| {
                    eprintln!("--delay-ms requires a value (milliseconds)");
                    std::process::exit(2);
                });
                delay_ms = val.parse().unwrap_or_else(|_| {
                    eprintln!("Invalid --delay-ms value: {}", val);
                    std::process::exit(2);
                });
            }
            _ => {
                eprintln!("Unknown arg: {}", arg);
                eprintln!("Usage: ccdemo [--delay-ms N] < input.txt");
                std::process::exit(2);
            }
        }
    }

    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() || input.trim().is_empty() {
        eprintln!("Provide text via stdin. Example:");
        eprintln!("  cat your_lyrics.txt | ccdemo --delay-ms 150");
        return;
    }

    let delay = Duration::from_millis(delay_ms);
    let mut first = true;
    for word in input.split_whitespace() {
        if !first {
            print!(" ");
        }
        first = false;
        print!("{}", word);
        io::stdout().flush().ok();
        sleep(delay);
    }
    println!();
}
