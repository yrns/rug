use rug::{Eval, Repl};

struct Echo;

impl Eval for Echo {}

fn main() {
    let _ = Repl::new(Echo).run();
}
