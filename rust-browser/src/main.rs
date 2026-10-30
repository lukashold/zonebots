use std::io::{self, BufRead, Write};
fn main() {
    for line in io::stdin().lock().lines() {
        match line {
            Ok(l) => {
                let result = zonebots_rust::request(l.as_bytes());
                println!("{}", String::from_utf8_lossy(&result));
                io::stdout().flush().unwrap();
            }
            Err(_) => break,
        }
    }
}
