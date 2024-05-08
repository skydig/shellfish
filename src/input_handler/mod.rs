#[cfg(feature = "rustyline")]
use rustyline::error::ReadlineError;
use std::io::{self, stdin, stdout, Write};

/// A trait for anything that can be used to gain user input
pub trait InputHandler {
    /// Reads user input
    fn read(&mut self, prompt: &str) -> io::Result<InputResult>;
}

pub enum InputResult {
    Interrupted,
    EOF,
    S(String),
}

/// Uses simple `std::io` methods to read
pub struct IO;

impl InputHandler for IO {
    fn read(&mut self, prompt: &str) -> io::Result<InputResult> {
        // Print prompt
        print!("{}", prompt);
        stdout().flush()?;

        // Read text
        let mut buffer = String::new();
        stdin().read_line(&mut buffer)?;

        Ok(InputResult::S(buffer))
    }
}

#[cfg(feature = "rustyline")]
impl<H, I> InputHandler for rustyline::Editor<H, I>
where
    H: rustyline::Helper,
    I: rustyline::history::History,
{
    fn read(&mut self, prompt: &str) -> io::Result<InputResult> {
        match self.readline(prompt) {
            Ok(o) => {
                self.add_history_entry(&o)
                    .map_err(convert_rustyline_to_io)?;
                Ok(InputResult::S(o))
            }
            Err(ReadlineError::Eof) => Ok(InputResult::EOF),
            Err(ReadlineError::Interrupted) => Ok(InputResult::Interrupted),
            Err(e) => Err(convert_rustyline_to_io(e)),
        }
    }
}

#[cfg(feature = "rustyline")]
fn convert_rustyline_to_io(e: ReadlineError) -> io::Error {
    match e {
        ReadlineError::Io(e) => e,
        ReadlineError::Errno(e) => e.into(),
        e => io::Error::new(io::ErrorKind::Interrupted, e),
    }
}
