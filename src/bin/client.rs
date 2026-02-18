use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let addr = args.get(1).map(|s| s.as_str()).unwrap_or("127.0.0.1:7878");

    println!("Connecting to {addr}...");
    let stream = match TcpStream::connect(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect: {e}");
            std::process::exit(1);
        }
    };
    println!("Connected!");

    let mut writer = stream.try_clone().expect("Failed to clone stream");
    let mut reader = BufReader::new(stream);
    let stdin = io::stdin();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => {
                println!("\nDisconnected from server.");
                break;
            }
            Ok(_) => {
                let msg = line.trim_end();
                if msg == "YOUR_TURN" {
                    print!("Your move (0-8): ");
                    io::stdout().flush().ok();
                    let mut input = String::new();
                    match stdin.lock().read_line(&mut input) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            if writer.write_all(input.as_bytes()).is_err() {
                                break;
                            }
                            writer.flush().ok();
                        }
                    }
                } else if !msg.is_empty() {
                    println!("{msg}");
                }
            }
        }
    }
}
