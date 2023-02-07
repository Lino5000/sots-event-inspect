use std::{
    error::Error,
    path::PathBuf,
};
use clap::Parser;

mod data;
mod yaml;
mod interface;
use interface::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File Path to extract from
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.path.is_file() {
        return Err("Please provide the path to the directory that contains the `.asset` files, not
                   a file.".into());
    } else if args.path.is_dir() {
        for (_id, event) in parse_event_data(args.path)? {
            println!("{}", event);
        }
    } else if !args.path.try_exists()? {
        return Err(format!("The file `{}` does not exist.", args.path.display()).into());
    } else {
        return Err("An unknown error occured".into());
    }

    Ok(())
}
