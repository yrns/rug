use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::{config::Config, Editor};

pub trait Eval {
    fn eval(&mut self, s: &str) -> Result<String> {
        Ok(s.to_owned())
    }
}

pub struct Repl<T> {
    editor: Editor<()>,
    eval: T,
}

impl<T: Eval> Repl<T> {
    pub fn new(eval: T) -> Self {
        let mut editor =
            Editor::<()>::with_config(Config::builder().auto_add_history(true).build());

        if editor.load_history("history.txt").is_err() {
            println!("No previous history.");
        }

        Self { editor, eval }
    }

    pub fn run(&mut self) {
        loop {
            let readline = self.editor.readline("user> ");
            match readline {
                Ok(line) => match self.eval.eval(&line) {
                    Ok(res) => println!("{}", res),
                    Err(err) => eprintln!("{}", err),
                },
                Err(ReadlineError::Interrupted) => break,
                Err(ReadlineError::Eof) => break,
                Err(_err) => break,
            }
        }

        println!("history is empty: {}", self.editor.history().is_empty());
        self.editor.save_history("history.txt").unwrap();
    }
}
