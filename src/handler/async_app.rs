use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use async_trait::async_trait;
use yansi::Paint;

use super::{AsyncHandler, CommandLineHandler};
use crate::command::CommandType;
use crate::Command;

/// Shellfish's CLI handler. This is helpful for when you want to parse
/// input from the command line, rather than in an interactive case.
///
/// The main differences are:
///  * It expects the binary name to be first
///  * Aswell as `help` one can use `--help`
#[derive(Default, Clone, Eq, PartialEq)]
pub struct DefaultAsyncCLIHandler {
    pub proj_name: Option<String>,
}

impl CommandLineHandler for DefaultAsyncCLIHandler {
    fn get_cache(&self) -> Option<PathBuf> {
        let mut path = home::home_dir()?;
        #[cfg(target_os = "windows")]
        {
            path.push("AppData");
            path.push("Local");
        }
        #[cfg(target_os = "linux")]
        {
            path.push(".cache");
        }
        #[cfg(target_os = "macos")]
        {
            path.push("Library Support");
        }
        path.push(self.proj_name.as_ref().unwrap_or(&env::args().next()?));
        path.push("shellfish.json");
        Some(path)
    }
}

#[async_trait]
impl<T: Send> AsyncHandler<T> for DefaultAsyncCLIHandler {
    async fn handle_async(
        &self,
        line: Vec<String>,
        commands: &HashMap<&str, Command<T>>,
        state: &mut T,
        description: &str,
    ) -> bool {
        if let Some(command) = line.get(1) {
            match command.as_str() {
                "quit" | "exit" | "--quit" | "--exit" => return true,
                "help" | "--help" => {
                    // Print the binary name
                    println!("{}", line[0]);

                    // Description
                    println!("{}", description);

                    // Usage section
                    println!("USAGE:");
                    println!(
                        "    {} [SUBCOMMAND]",
                        self.proj_name.as_ref().unwrap_or(&line[0])
                    );
                    println!();

                    // Subcommand section
                    println!("Where [SUBCOMMAND] is one of:");

                    // Create a list of commands
                    let mut cmd_help = HashMap::new();
                    let mut cmd_len = 4;

                    // Add the built ins
                    cmd_help.insert("help", "displays help information.");
                    cmd_help.insert(
                        "quit",
                        "deletes all temporary state information.",
                    );
                    cmd_help.insert(
                        "exit",
                        "deletes all temporary state information.",
                    );

                    // Add the user defined
                    for (name, command) in commands {
                        cmd_help.insert(name, &command.help);
                        cmd_len = cmd_len.max(name.len());
                    }

                    // Go through the built in subcommands
                    for (name, command) in cmd_help {
                        println!(
                            "    {:<width$}{}",
                            name,
                            command,
                            width = cmd_len + 5
                        );
                    }
                }
                _ => {
                    let command = commands.get(command as &str);
                    let line =
                        line[1..].iter().map(|x| x.to_string()).collect();

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
                                "{}",
                                Paint::red(format!(
                                    "Command not found: {}",
                                    line[0]
                                )),
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
