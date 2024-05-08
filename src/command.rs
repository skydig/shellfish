use std::error::Error;
#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

// NOTE: Taken from StackOverflow
/// Use this macro to wrap an asynchronous function so that it can be used
/// as a function pointer.
///
/// This is used with async [`Command`s](Command).
///
/// The first argument is a Type, which is the state. The second is the
/// async function.
#[macro_export]
#[cfg_attr(nightly, doc(cfg(feature = "async")))]
macro_rules! async_fn {
    ($state:ty, $inc:expr) => {{
       // I think the error message referred to here is spurious, but why take a chance?
       fn rustc_complains_if_this_name_conflicts_with_the_environment_even_though_its_probably_fine(
           state: &mut $state,
           args: Vec<String>
       ) -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Result<(), Box<dyn ::std::error::Error>>> + Send + '_ >> {
            Box::pin($inc(state, args))
        }
        rustc_complains_if_this_name_conflicts_with_the_environment_even_though_its_probably_fine
    }}
}

#[derive(Clone)]
pub struct Command<T> {
    /// The function pointer which this links to.
    pub command: CommandType<T>,
    /// A help string, should be less than 80 characters. For example, if it
    /// was an `echo` command:
    /// ```txt
    /// prints the arguments to the output.
    /// ```
    pub help: String,
}

impl<T> Command<T> {
    /// Creates a new `Command`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shellfish::*;
    /// use std::error::Error;
    ///
    /// fn greet(_state: &mut (), args: Vec<String>) -> Result<(), Box<dyn Error>> {
    ///     //--snip--
    ///     # Ok(())
    /// }
    ///
    /// fn main() {
    ///     // Creates a shell
    ///     let mut shell = Shell::new((), "[Shell]-$");
    ///
    ///     // Creates a command
    ///     shell.commands.insert(
    ///         "greet",
    ///         Command::new("greets_you".to_string(), greet),
    ///     );
    /// }
    /// ```
    pub fn new(help: String, command: CommandFn<T>) -> Self {
        Self {
            command: CommandType::Sync(command),
            help,
        }
    }

    /// Creates a new asynchronous `Command`.
    ///
    /// It is important to note that you have to call
    /// [`async_fn!`](async_fn!) to prepare a function for it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shellfish::*;
    /// use std::error::Error;
    ///
    /// async fn greet(_state: &mut (), args: Vec<String>) -> Result<(), Box<dyn Error>> {
    ///     //--snip--
    ///     # Ok(())
    /// }
    ///
    /// async fn async_main() {
    ///     // Creates a shell
    ///     let mut shell = Shell::new((), "[Shell]-$");
    ///
    ///     // Creates a command
    ///     shell.commands.insert(
    ///         "greet",
    ///         Command::new_async("greets_you".to_string(), async_fn!((), greet)),
    ///     );
    /// }
    /// ```
    #[cfg(feature = "async")]
    pub fn new_async(help: String, command: AsyncCommandFn<T>) -> Self {
        Self {
            command: CommandType::Async(command),
            help,
        }
    }
}

/// Stores a function for a [`Command`](Command).
///
/// It requires the function returns a `Result<(), Box<dyn Error>>`.
pub type CommandFn<T> = fn(&mut T, Vec<String>) -> Result<(), Box<dyn Error>>;

/// Stores an asynchronous function for a [`Command`](Command).
///
/// It requires the function returns a `Result<(), Box<dyn Error>>`.
///
/// To prepare for this you have to use the [`async_fn!`](async_fn!)
/// macro to prepare the function.
#[cfg(feature = "async")]
pub type AsyncCommandFn<T> = fn(
    &mut T,
    Vec<String>,
) -> Pin<
    Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + '_>,
>;

/// Command type specifies what type of command this is, namely wether it
/// is async or not.
#[derive(Clone)]
pub enum CommandType<T> {
    Sync(CommandFn<T>),
    #[cfg(feature = "async")]
    Async(AsyncCommandFn<T>),
}
