use std::io::stdin;

use parser::Bril;
use cfg::BrilCFG;

mod parser;
mod cfg;
mod dce;

fn main() {
    let mut s = String::new();
    for line in stdin().lines() {
        if let Ok(res) = line {
            s.push_str(&res);
        }
    }
    let bril: Bril = serde_json::from_str(&s).unwrap();
    let mut cfg = BrilCFG::new(bril);
    cfg.parse_blocks();
    for block in cfg.blocks {
        println!("{block}");
    }
}

