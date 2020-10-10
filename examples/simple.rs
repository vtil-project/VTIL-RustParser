use std::env;
use vtil_parser::{Result, VTILReader};

fn main() -> Result<()> {
    let mut argv = env::args();
    let routine = VTILReader::from_path(argv.nth(1).unwrap())?;
    println!(
        "The architecture of this VTIL file is: {:?}",
        routine.header().arch_id()
    );
    Ok(())
}
