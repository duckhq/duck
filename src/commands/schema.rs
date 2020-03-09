use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

use duck::DuckResult;

#[derive(StructOpt, Debug)]
pub struct Arguments {
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,
}

/// Executes the run command
pub async fn execute(args: Arguments) -> DuckResult<()> {
    let mut file = File::create(args.output)?;
    file.write_all(duck::get_schema().as_bytes())?;
    Ok(())
}
