//! # App
//!
//! Apps allow you to create commamd line argument parsing for your shellfish
//! commands. Basically, you define your commands as normal and call
//! [`.run_args`](App::run_args) on app. State is also saved, and only
//! deleted when given `exit` or `quit`.
//!
//! **Note: Normal handlers won't work as they MUST assume that
//! the first argument is that of the binaries name.**

use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::io::{Read, Write};

#[cfg(feature = "async-std")]
use async_std::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::handler::app::{CommandLineHandler, DefaultCommandLineHandler};
#[cfg(feature = "async")]
use crate::handler::async_app::DefaultAsyncCLIHandler;
use crate::*;

/// See the module level dicumentation. Note `App` closely mirrors state and
/// so can be created from it (given the right trait bounds)
pub struct App<
    'b,
    T: Serialize + for<'a> Deserialize<'a>,
    H: CommandLineHandler,
> {
    pub commands: HashMap<&'b str, Command<T>>,
    pub state: T,
    pub handler: H,
    pub description: String,
}

impl<
        'f: 't,
        't,
        T: Serialize + for<'a> Deserialize<'a>,
        M: Display,
        H: Handler<T>,
        I: InputHandler,
    > TryFrom<Shell<'f, T, M, H, I>> for App<'t, T, DefaultCommandLineHandler>
{
    type Error = Box<dyn Error>;

    fn try_from(shell: Shell<'f, T, M, H, I>) -> Result<Self, Box<dyn Error>> {
        let mut this = Self {
            commands: shell.commands,
            state: shell.state,
            handler: DefaultCommandLineHandler { proj_name: None },
            description: shell.description,
        };
        this.load_cache()?;
        Ok(this)
    }
}

#[cfg(feature = "async")]
impl<'f: 't, 't, T: Serialize + for<'a> Deserialize<'a> + Send>
    App<'t, T, DefaultAsyncCLIHandler>
{
    pub fn try_from_async<M: Display, H: AsyncHandler<T>, I: InputHandler>(
        shell: Shell<'f, T, M, H, I>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut this = Self {
            commands: shell.commands,
            state: shell.state,
            handler: DefaultAsyncCLIHandler { proj_name: None },
            description: shell.description,
        };
        this.load_cache()?;
        Ok(this)
    }
}

impl<T: Serialize + for<'a> Deserialize<'a>>
    App<'_, T, DefaultCommandLineHandler>
{
    /// Creates a new shell
    pub fn new(state: T, project_name: String) -> Result<Self, Box<dyn Error>> {
        let mut this = Self {
            commands: HashMap::new(),
            state,
            handler: DefaultCommandLineHandler {
                proj_name: Some(project_name),
            },
            description: String::new(),
        };
        this.load_cache()?;
        Ok(this)
    }
}

impl<T: Serialize + for<'a> Deserialize<'a>, H: CommandLineHandler>
    App<'_, T, H>
{
    /// Creates a new shell with the given handler.
    ///
    /// **Note that this should be a handler which can deal with cli
    /// arguments, as in the FIRST ARGUMENT is the BINARY name.**
    pub fn new_with_handler(
        state: T,
        handler: H,
    ) -> Result<Self, Box<dyn Error>> {
        let mut this = Self {
            commands: HashMap::new(),
            state,
            handler,
            description: String::new(),
        };
        this.load_cache()?;
        Ok(this)
    }

    /// Loads the state from cache.
    pub fn load_cache(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(cache) = self.handler.get_cache() {
            // Try and open the file
            if let Ok(mut file) = fs::File::open(cache) {
                // Get the string
                let mut string = String::new();
                file.read_to_string(&mut string)?;

                // Parse it with serde json
                self.state = serde_json::from_str(&string)?;
            }
        }

        Ok(())
    }
}

impl<
        T: Serialize + for<'a> Deserialize<'a>,
        H: CommandLineHandler + Handler<T>,
    > App<'_, T, H>
{
    /// Handles an vec of strings, like environment arguments.
    ///
    /// Returns a bool on wether we have 'quit' or not
    pub fn run_vec(&mut self, vec: Vec<String>) -> std::io::Result<bool> {
        let result = self.handler.handle(
            vec,
            &self.commands,
            &mut self.state,
            &self.description,
        );

        // Do stuff with the cache
        match result {
            // Delete the cache
            true => {
                if let Some(cache) = self.handler.get_cache() {
                    fs::remove_file(cache)?;
                }
            }
            // Write the cache
            false => {
                if let Some(cache) = self.handler.get_cache() {
                    // Create the dir
                    if let Some(dir) = cache.parent() {
                        fs::create_dir_all(dir)?;
                    }

                    // Create the file
                    let mut file = fs::File::create(cache)?;
                    file.write_all(
                        serde_json::to_string(&self.state)?.as_bytes(),
                    )?;
                }
            }
        }
        Ok(result)
    }

    /// Runs from the env args
    pub fn run_args(&mut self) -> std::io::Result<bool> {
        self.run_vec(env::args().collect())
    }
}

#[cfg(feature = "async")]
impl<
        T: Serialize + for<'a> Deserialize<'a> + Send,
        H: CommandLineHandler + AsyncHandler<T>,
    > App<'_, T, H>
{
    /// Handles an vec of strings, like environment arguments.
    ///
    /// Returns a bool on wether we have 'quit' or not
    pub async fn run_vec_async(
        &mut self,
        vec: Vec<String>,
    ) -> std::io::Result<bool> {
        let result = self
            .handler
            .handle_async(
                vec,
                &self.commands,
                &mut self.state,
                &self.description,
            )
            .await;

        // Do stuff with the cache
        match result {
            // Delete the cache
            true => {
                if let Some(cache) = self.handler.get_cache() {
                    cfg_if::cfg_if! {
                        if #[cfg(feature = "async-std")] {
                            async_std::fs::remove_file(cache).await?;
                        } else if #[cfg(feature = "tokio")] {
                            tokio::fs::remove_file(cache).await?;
                        } else {
                            fs::remove_file(cache)?;
                        }
                    }
                }
            }
            // Write the cache
            false => {
                if let Some(cache) = self.handler.get_cache() {
                    cfg_if::cfg_if! {
                        if #[cfg(feature = "async-std")] {
                            // Create the dir
                            if let Some(dir) = cache.parent() {
                                async_std::fs::create_dir_all(dir).await?;
                            }

                            // Create the file
                            let mut file = async_std::fs::File::create(cache).await?;
                            file.write_all(
                                serde_json::to_string(&self.state)?.as_bytes(),
                            ).await?;
                        } else if #[cfg(feature = "tokio")] {
                            // Create the dir
                            if let Some(dir) = cache.parent() {
                                tokio::fs::create_dir_all(dir).await?;
                            }

                            // Create the file
                            let mut file = tokio::fs::File::create(cache).await?;
                            file.write_all(
                                serde_json::to_string(&self.state)?.as_bytes(),
                            ).await?;
                        } else {
                            // Create the dir
                            if let Some(dir) = cache.parent() {
                                fs::create_dir_all(dir)?;
                            }

                            // Create the file
                            let mut file = fs::File::create(cache)?;
                            file.write_all(
                                serde_json::to_string(&self.state)?.as_bytes(),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    /// Runs from the env args
    pub async fn run_args_async(&mut self) -> std::io::Result<bool> {
        self.run_vec_async(env::args().collect()).await
    }
}
