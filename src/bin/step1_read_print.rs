use anyhow::{anyhow, Result};
use lassie::{Expr, ExprType};
//use lexpr::{parse::Error, Value};
use rug::{Eval, Repl};

struct Parser;

fn wrap(mut this: Expr, s: &str) -> Expr {
    this.prefix = None;
    Expr::new(
        None,
        ExprType::List(vec![Expr::new(None, ExprType::Symbol(s.to_string())), this]),
    )
}

fn expand(this: &mut Expr) {
    match this.expr {
        ExprType::List(ref mut a) | ExprType::Vector(ref mut a) | ExprType::Hashmap(ref mut a) => {
            for el in a.iter_mut() {
                expand(el)
            }
        }
        _ => (),
    };
    if let Some(prefix) = this.prefix.as_ref() {
        match prefix.as_str() {
            "'" => *this = wrap(this.clone(), "quote"),
            "~" => *this = wrap(this.clone(), "unquote"),
            "," => *this = wrap(this.clone(), "quasiquote"),
            "`" => *this = wrap(this.clone(), "quasiquote"),
            "~@" => *this = wrap(this.clone(), "splice-unquote"),
            "@" => *this = wrap(this.clone(), "deref"),
            "^" => *this = wrap(this.clone(), "with-meta"),
            _ => (),
        }
    }
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
            .map(|mut exprs| {
                exprs
                    .iter_mut()
                    .map(|e| {
                        expand(e);
                        e.to_string()
                    })
                    .collect()
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
