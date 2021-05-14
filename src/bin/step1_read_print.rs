use anyhow::{anyhow, Result};
use lassie::{Expr, ExprType};
//use lexpr::{parse::Error, Value};
use rug::{Eval, Repl};

struct Parser;

fn symbol(s: &str) -> Expr {
    Expr::new(None, ExprType::Symbol(s.to_string()))
}

fn list(v: Vec<Expr>) -> Expr {
    Expr::new(None, ExprType::List(v))
}

fn empty() -> Expr {
    list(vec![])
}

fn wrap(mut this: Expr, s: &str) -> Expr {
    this.prefix = None;
    list(vec![symbol(s), this])
}

// return the first element of the list (if a list)
fn unwrap(this: Expr) -> Vec<Expr> {
    match this.expr {
        ExprType::List(list) => list,
        // or just error
        _ => vec![this],
    }
}

fn expand<I: Iterator<Item = Expr>>(mut this: Expr, mut next: I) -> Expr {
    //println!("expanding: {:?}", this);

    match this.expr {
        ExprType::List(ref mut a) | ExprType::Vector(ref mut a) | ExprType::Hashmap(ref mut a) => {
            let mut res = Vec::new();
            {
                // pass the remaining elements to each call to expand
                // so the expansion can consume multiple elements
                // (with-meta)
                let mut next = a.drain(..);
                while let Some(e) = next.next() {
                    res.push(expand(e, &mut next))
                }
                // drop next
            }
            *a = res
        }
        _ => (),
    };
    if let Some(prefix) = this.prefix.as_ref() {
        match prefix.as_str() {
            "'" => this = wrap(this.clone(), "quote"),
            "~" => this = wrap(this.clone(), "unquote"),
            "," => this = wrap(this.clone(), "quasiquote"),
            "`" => this = wrap(this.clone(), "quasiquote"),
            "~@" => this = wrap(this.clone(), "splice-unquote"),
            "@" => this = wrap(this.clone(), "deref"),
            "^" => {
                this.prefix = None;
                this = list(vec![
                    symbol("with-meta"),
                    // this should be an error if empty
                    next.next().unwrap_or_else(|| empty()),
                    this.clone(),
                ])
            }
            _ => (),
        }
    }
    this
}

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
            // wrap every expression in a list so expansion works, and
            // unwrap before printing
            .map(|exprs| {
                unwrap(expand(list(exprs), std::iter::empty()))
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
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
