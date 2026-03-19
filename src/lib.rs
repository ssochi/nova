pub mod bytecode;
pub mod cli;
pub mod config;
pub mod driver;
pub mod frontend;
pub mod runtime;
pub mod semantic;
pub mod source;

use std::io::{self, Write};

pub fn run_cli<I>(args: I) -> Result<String, driver::DriverError>
where
    I: IntoIterator<Item = String>,
{
    driver::run_cli(args)
}

pub fn run_and_print<I>(args: I) -> Result<(), driver::DriverError>
where
    I: IntoIterator<Item = String>,
{
    let output = run_cli(args)?;
    if !output.is_empty() {
        io::stdout()
            .write_all(output.as_bytes())
            .map_err(|error| driver::DriverError::Io(error.to_string()))?;
    }
    Ok(())
}
