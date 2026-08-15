#[cfg(feature = "cli")]
mod cli;



fn main() {
    #[cfg(not(feature = "cli"))]
    handle_pipe();

    #[cfg(feature = "cli")]
    cli::handle_cli();
}

#[cfg(not(feature = "cli"))]
fn handle_pipe() {
    use std::io::{self, BufReader, BufWriter};


    let reader = BufReader::new(io::stdin());
    let writer = BufWriter::new(io::stdout());
    let result = fb2epub::convert(
        reader,
        writer,
        None, // styles
        None, // metadata
        false, // suspend_error_messages
        false, // debug
    );

    if let Err(err) = result {
        eprintln!("{err}");
    }
}
