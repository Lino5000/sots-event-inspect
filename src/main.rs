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
pub struct Args {
    /// File Path to extract from
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.path.is_dir() {
        let mut app = App::new(args)?;
        app.run()?;
        Ok(())
    } else {
        Err(if args.path.is_file() {
                "Please provide the path to the directory that contains the `.asset` files, not a file.".into()
            } else if !args.path.try_exists()? {
                format!("The file `{}` does not exist.", args.path.display()).into()
            } else {
                "An unknown error occured".into()
            })
    }
}
