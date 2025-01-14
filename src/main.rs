use std::io::stdin;

use parser::Bril;

mod parser;
mod cfg;

fn main() {
    let mut s = String::new();
    for line in stdin().lines() {
        if let Ok(res) = line {
            s.push_str(&res);
        }
    }
    let bril: Bril = serde_json::from_str(&s).unwrap();
    println!("bril {bril:?}");
}

