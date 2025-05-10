use color_eyre::eyre;
use wabznasm::repl;

fn main() -> Result<(), eyre::Report> {
    // Set up colorful error reporting
    color_eyre::install()?;

    // JIT and IR dump functionality has been removed.
    repl::run()
}
