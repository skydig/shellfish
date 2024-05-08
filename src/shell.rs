use std::collections::HashMap;
use std::fmt::Display;
use std::io;

#[cfg(feature = "rustyline")]
use thiserror::Error;
use yansi::Paint;

use crate::{
    input_handler::{InputResult, IO},
    *,
};

/// A shell represents a shell for editing commands in.
///
/// Here are the generics:
///  * T: The state.
///  * M: The prompt. Can be anything that can be printed.
///  * H: The handler. Should implement either `Handler` or `AsyncHandler`, or
///    no functionality is present.
///  * I: The input handler. Must implement [`InputHandler`]
#[derive(Clone)]
pub struct Shell<'a, T, M: Display, H, I: InputHandler> {
    /// The shell prompt.
    ///
    /// It can be anything which implements Display and can therefore be
    /// printed (This allows for prompts that change with the state.)
    pub prompt: M,
    /// This is a list of commands for the shell. The hashmap key is the
    /// name of the command (ie `"greet"`) and the value is a wrapper
    /// to the function it corresponds to (as well as help information.)
    pub commands: HashMap<&'a str, Command<T>>,
    /// This is the state of the shell. This stores any values that you
    /// need to be persisted over multiple shell commands. For example
    /// it may be a simple counter or maybe a session ID.
    pub state: T,
    /// This is the handler for commands. See the [`Handler`](crate::Handler)
    /// documentation for more.
    pub handler: H,
    /// This is the description of the shell as a whole. This is displayed when
    /// requesting help information.
    pub description: String,
    /// The input method
    pub input_handler: I,
}

impl<'a, T, M: Display> Shell<'a, T, M, handler::DefaultHandler, IO> {
    /// Creates a new shell
    ///
    /// If you have the `rustyline` feature enabled use [`new_with_handler`]
    /// to use the `rustyline` editor.
    pub fn new(state: T, prompt: M) -> Self {
        Shell {
            prompt,
            commands: HashMap::new(),
            state,
            handler: handler::DefaultHandler(),
            description: String::new(),
            input_handler: IO,
        }
    }
}

#[cfg(feature = "async")]
#[cfg_attr(nightly, doc(cfg(feature = "async")))]
impl<'a, T, M: Display> Shell<'a, T, M, handler::DefaultAsyncHandler, IO> {
    /// Creates a new shell
    ///
    /// If you have the `rustyline` feature enabled use [`new_with_async_handler`]
    /// to use the `rustyline` editor.
    pub fn new_async(state: T, prompt: M) -> Self {
        Shell {
            prompt,
            commands: HashMap::new(),
            state,
            handler: handler::DefaultAsyncHandler(),
            description: String::new(),
            input_handler: IO,
        }
    }
}

impl<'a, T, M: Display, H: Handler<T>, I: InputHandler> Shell<'a, T, M, H, I> {
    /// Creates a new shell with the given handler.
    pub fn new_with_handler(
        state: T,
        prompt: M,
        handler: H,
        input_handler: I,
    ) -> Self {
        Shell {
            prompt,
            commands: HashMap::new(),
            state,
            handler,
            description: String::new(),
            input_handler,
        }
    }

    /// Starts running the shell
    pub fn run(&mut self) -> io::Result<()> {
        '_shell: loop {
            // Read a line
            let line =
                match self.input_handler.read(&self.prompt.to_string())? {
                    InputResult::S(line) => line,
                    InputResult::Interrupted => continue '_shell,
                    InputResult::EOF => break '_shell,
                };

            // Runs the line
            match Self::unescape(line.trim()) {
                Ok(line) => {
                    if self.handler.handle(
                        line,
                        &self.commands,
                        &mut self.state,
                        &self.description,
                    ) {
                        break '_shell;
                    }
                }
                Err(e) => eprintln!("{}", Paint::red(e.to_string().as_str())),
            }
        }
        Ok(())
    }
}

#[cfg(feature = "async")]
#[cfg_attr(nightly, doc(cfg(feature = "async")))]
impl<'a, T: Send, M: Display, H: AsyncHandler<T>, I: InputHandler>
    Shell<'a, T, M, H, I>
{
    /// Creates a new shell with the given handler.
    pub fn new_with_async_handler(
        state: T,
        prompt: M,
        handler: H,
        input_handler: I,
    ) -> Self {
        Shell {
            prompt,
            commands: HashMap::new(),
            state,
            handler,
            description: String::new(),
            input_handler,
        }
    }

    /// Starts running the shell
    pub async fn run_async(&mut self) -> io::Result<()> {
        '_shell: loop {
            // Read a line
            let line =
                match self.input_handler.read(&self.prompt.to_string())? {
                    InputResult::S(line) => line,
                    InputResult::Interrupted => continue '_shell,
                    InputResult::EOF => break '_shell,
                };

            // Runs the line
            match Self::unescape(line.trim()) {
                Ok(line) => {
                    if self
                        .handler
                        .handle_async(
                            line,
                            &self.commands,
                            &mut self.state,
                            &self.description,
                        )
                        .await
                    {
                        break '_shell;
                    }
                }
                Err(e) => eprintln!("{}", Paint::red(e.to_string())),
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum UnescapeError {
    #[error("unhandled escape sequence \\{0}")]
    UnhandledEscapeSequence(char),
    #[error("unclosed quotes")]
    UnclosedQuotes,
}

impl<'a, T, M: Display, H, I: InputHandler> Shell<'a, T, M, H, I> {
    /// Unescapes a line and gets the arguments.
    fn unescape(command: &str) -> Result<Vec<String>, UnescapeError> {
        // Create a vec to store the split int.
        let mut vec = vec![String::new()];

        // Are we in an escape sequence?
        let mut escape = false;

        // Are we in a string?
        let mut string = false;

        // Go through each char in the string
        for c in command.chars() {
            let segment = vec.last_mut().unwrap();
            if escape {
                match c {
                    '\\' => segment.push('\\'),
                    ' ' if !string => segment.push(' '),
                    'n' => segment.push('\n'),
                    'r' => segment.push('\r'),
                    't' => segment.push('\t'),
                    '"' => segment.push('"'),
                    _ => return Err(UnescapeError::UnhandledEscapeSequence(c)),
                }
                escape = false;
            } else {
                match c {
                    '\\' => escape = true,
                    '"' => string = !string,
                    ' ' if string => segment.push(c),
                    ' ' if !string => vec.push(String::new()),
                    _ => segment.push(c),
                }
            }
        }

        if string {
            return Err(UnescapeError::UnclosedQuotes);
        }

        if vec.len() == 1 && vec[0].is_empty() {
            vec.clear();
        }

        Ok(vec)
    }
}
