# wordexp-rs

[![Docs](https://docs.rs/wordexp/badge.svg)](https://docs.rs/wordexp/)

Rust wrapper around wordexp c library

## Examples
```rust
use wordexp::{wordexp, Wordexp};

std::env::set_var("HOME", "/home/wordexp");
match wordexp("~/", Wordexp::new(0), 0) {
  Ok(wexp) => for exp in wexp {
    println!("exp: {}", exp);
  },
  Err(e) => eprintln!("Error: {}", e);
}
/// output:
/// exp: /home/wordexp/
```
