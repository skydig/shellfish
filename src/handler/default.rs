//use std::collections::HashMap;
use indexmap::IndexMap;

use yansi::Paint;

use crate::command::CommandType;
use crate::Command;
/// A handler lets you change how commands are run. They also let you
/// change the shell built-ins. A handler takes a Vec<String> as
/// input, and return a bool. A handler also takes a HashMap<String, Commands>,
/// so it knows what commands it can run. Likewise, the state is also given.
///
/// The bool sent in return is wether or not this command should quit the
/// shell. For example, in default shellfish, `true` is only every returned
/// when the commands `quit` or `exit` are given.
///
/// For nearly every use case
/// the default handler should be enough. If in doubt, you can create your
/// own. I would recommend looking at shellfish's source code for an example
pub trait Handler<T> {
    fn handle(
        &self,
        args: Vec<String>,
        commands: &IndexMap<&str, Command<T>>,
        state: &mut T,
        description: &str,
    ) -> bool;
}

/// Shellfish's default handler. This handler is pretty simple, given the
/// only special options are `help`, `quit` and `exit`.
#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct DefaultHandler();

impl<T> Handler<T> for DefaultHandler {
    fn handle(
        &self,
        line: Vec<String>,
        commands: &IndexMap<&str, Command<T>>,
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
                    let command = commands.get(&*line[0]);

                    // Checks if we got it
                    match command {
                        Some(command) => {
                            if let Err(e) = match command.command {
                                CommandType::Sync(c) => c(state, line),
                                #[cfg(feature = "async")]
                                CommandType::Async(_) => {
                                    eprintln!("{}", Paint::red("Async commands cannot be run in sync shells."));
                                    Ok(())
                                }
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
