use std::collections::HashMap;

use async_trait::async_trait;
use yansi::Paint;

use crate::command::CommandType;
use crate::Command;

/// Async handler lets you run asynchronous commands. It also requires the
/// shell to be run in asynchronous mode to support it.
#[async_trait]
pub trait AsyncHandler<T: Send> {
    async fn handle_async(
        &self,
        args: Vec<String>,
        commands: &HashMap<&str, Command<T>>,
        state: &mut T,
        description: &str,
    ) -> bool;
}

/// Shellfish's default async handler. This handler is pretty simple, given
/// the only built in commands are `help`, `quit` and `exit`.
#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct DefaultAsyncHandler();

#[async_trait]
impl<T: Send> AsyncHandler<T> for DefaultAsyncHandler {
    async fn handle_async(
        &self,
        line: Vec<String>,
        commands: &HashMap<&str, Command<T>>,
        state: &mut T,
        description: &str,
    ) -> bool {
        if let Some(command) = line.get(0) {
            // Add some padding.
            println!();

            match command.as_str() {
                "quit" | "exit" => return true,
                "help" => {
                    println!("{}", description);

                    // Print information about built-in commands
                    println!("    help - displays help information.");
                    println!("    quit - quits the shell.");
                    println!("    exit - exits the shell.");
                    for (name, command) in commands {
                        println!("    {} - {}", name, command.help);
                    }
                }
                _ => {
                    // Attempt to find the command
                    let command = commands.get(&line[0] as &str);

                    // Checks if we got it
                    match command {
                        Some(command) => {
                            if let Err(e) = match command.command {
                                CommandType::Sync(c) => c(state, line),
                                #[cfg(feature = "async")]
                                CommandType::Async(a) => a(state, line).await,
                            } {
                                eprintln!("{}", Paint::red(format!("Command exited unsuccessfully:\n{}\n({:?})", &e, &e)))
                            }
                        }
                        None => {
                            eprintln!(
                                "{} {}",
                                Paint::red("Command not found:"),
                                line[0]
                            )
                        }
                    }
                }
            }

            // Padding
            println!();
        }
        false
    }
}
