use anyhow::{anyhow, Result};
//use lexpr::{parse::Error, Value};
use rug::{Eval, Repl};

struct Parser;

impl Eval for Parser {
    fn eval(&mut self, s: &str) -> Result<String> {
        // lexpr doesn't handle these test cases:
        // let s = match s.trim() {
        //     "(1 2, 3,,,,),," => &"(1 2 3)",
        //     _ => s,
        // };
        //let expr = lexpr::from_str(&s)?;

        //println!("input: {}", &s);

        lassie::parse::exprs(&s)
            .map(|exprs| exprs.iter().map(|e| e.to_string()).collect())
            .map_err(|e| {
                let i = e.location.offset;
                let got = if i >= s.len() {
                    "EOF".to_string()
                } else {
                    s.get(i..)
                        .and_then(|s| s.lines().next())
                        .map(|a| a.to_owned())
                        .unwrap_or_else(|| "nothing".to_string())
                };
                anyhow!(
                    "error: got: {} at: {}:{} expected: {}",
                    got,
                    e.location.line,
                    e.location.column,
                    e.expected
                )
            })
    }
}

fn main() {
    let _ = Repl::new(Parser).run();
}
