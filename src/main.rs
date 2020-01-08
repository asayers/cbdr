mod analyze;
mod diff;
mod label;
mod pretty;
mod sample;
mod summarize;

use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    Sample(sample::Options),
    // Diff(diff::Options),
    // Pretty,
    // Summarize,
    Analyze(analyze::Options),
}

fn main() {
    env_logger::init();
    let result = match Subcommand::from_args() {
        Subcommand::Sample(opts) => sample::sample(opts),
        // Subcommand::Diff(opts) => diff::diff(opts),
        // Subcommand::Pretty => pretty::pretty(),
        // Subcommand::Summarize => summarize::summarize(),
        Subcommand::Analyze(opts) => analyze::analyze(opts),
    };
    match result {
        Ok(()) => (),
        Err(e) => {
            // Ignore EPIPE
            if let Some(e) = e.downcast_ref::<std::io::Error>() {
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    return;
                }
            }
            eprintln!("Error: {}", e);
            std::process::exit(1)
        }
    }
}
