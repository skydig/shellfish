/// Creates a [`Command`](crate::Command) which takes a [`clap::Parser`] as
/// input.
///
/// This macro creates a wrapper around synchronous or asynchronous functions to
/// automatically parse and pass the clap argument, and will
/// return an error to the shell if it cannot be suitably parsed.
///
/// ```
/// #[derive(Parser, Debug)]
/// #[clap(author, version, about)]
/// struct Args {
///     // - snip
/// }
///
/// shell.commands.insert("greet", clap_command!((), Args, greet));
/// shell.commands.insert("greet-async", clap_command!((), Args, async greet_async));
///
/// fn greet(_state: &mut (), args: Args) -> Result<(), Box<dyn std::error::Error>> {
///     // - snip
/// }
///
/// async fn greet_async(_state: &mut (), args: Args) -> Result<(), Box<std::io::Error>> {
///     // - snip
/// }
/// ```
///
/// NOTE: You need the `async` crate feature enabled for async
///       clap commands.
#[cfg_attr(nightly, doc(cfg(feature = "clap")))]
#[macro_export]
macro_rules! clap_command {
    ($state: ty, $clap: ty, async $inc: expr) => {{
        async fn func(
            state: &mut $state,
            args: Vec<String>,
        ) -> Result<(), Box<dyn ::std::error::Error>> {
            let parsed: $clap = match ::clap::Parser::try_parse_from(&args[..])
            {
                ::std::result::Result::Ok(o) => o,
                ::std::result::Result::Err(e)
                    if e.kind() == ::clap::error::ErrorKind::DisplayHelp =>
                {
                    e.print()?;
                    return Ok(());
                }
                ::std::result::Result::Err(e) => {
                    return std::result::Result::Err(e.into())
                }
            };
            $inc(state, parsed).await?;
            Ok(())
        }
        let command = $crate::Command::new_async(
            <$clap as ::clap::CommandFactory>::command()
                .get_about()
                .unwrap_or_default()
                .to_string(),
            $crate::async_fn!($state, func),
        );
        command
    }};
    ($state: ty, $clap: ty, $inc: expr) => {{
        fn func(
            state: &mut $state,
            args: Vec<String>,
        ) -> Result<(), Box<dyn ::std::error::Error>> {
            let parsed: $clap = match ::clap::Parser::try_parse_from(&args[..])
            {
                ::std::result::Result::Ok(o) => o,
                ::std::result::Result::Err(e)
                    if e.kind() == ::clap::error::ErrorKind::DisplayHelp =>
                {
                    e.print()?;
                    return Ok(());
                }
                ::std::result::Result::Err(e) => {
                    return std::result::Result::Err(e.into())
                }
            };
            $inc(state, parsed)?;
            Ok(())
        }
        let command = $crate::Command::new(
            <$clap as ::clap::CommandFactory>::command()
                .get_about()
                .unwrap_or_default()
                .to_string(),
            func,
        );
        command
    }};
}
