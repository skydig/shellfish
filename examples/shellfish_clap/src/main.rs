use async_std::io::ReadExt;
use clap::Parser;
use rustyline::DefaultEditor;
use shellfish::{clap_command, handler::DefaultAsyncHandler, Shell};
use std::path::PathBuf;

/// Simple command to greet a person
///
/// This command will greet the person based of a multitide
/// of option flags, see below.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct GreetArgs {
    /// Name of the person to greet
    name: String,

    /// Age of the person to greet
    #[clap(short, long)]
    age: Option<u8>,

    /// Whether to be formal or note
    #[clap(short, long)]
    formal: bool,
}

/// Opens and prints each input file
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct CatArgs {
    pub paths: Vec<PathBuf>,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a shell
    let mut shell = Shell::new_with_async_handler(
        (),
        "<[Shellfish Example]>-$ ",
        DefaultAsyncHandler::default(),
        DefaultEditor::new()?,
    );
    shell
        .commands
        .insert("greet", clap_command!((), GreetArgs, greet));
    shell
        .commands
        .insert("cat", clap_command!((), CatArgs, async cat));
    shell.run_async().await?;

    Ok(())
}

fn greet(
    _state: &mut (),
    args: GreetArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.formal {
        match args.age {
            Some(age) => {
                println!("Good day {}, you are {} years old.", args.name, age)
            }
            None => println!("Good day {}.", args.name),
        }
    } else {
        match args.age {
            Some(age) => {
                println!("Hi {}, you're {} years old.", args.name, age)
            }
            None => println!("Hi {}!", args.name),
        }
    }

    Ok(())
}

/// Asynchronously reads a file
async fn cat(_state: &mut (), args: CatArgs) -> Result<(), std::io::Error> {
    use async_std::fs;

    for file in args.paths {
        let mut contents = String::new();
        let mut file = fs::File::open(file).await?;
        file.read_to_string(&mut contents).await?;
        println!("{}", contents);
    }

    Ok(())
}
