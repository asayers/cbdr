mod diff;
mod limit;
mod pretty;
mod sample;

use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    Diff(diff::Options),
    Sample(sample::Options),
    Pretty,
    Limit(limit::Options),
}

fn main() {
    env_logger::init();
    let result = match Subcommand::from_args() {
        Subcommand::Diff(opts) => diff::diff(opts),
        Subcommand::Sample(opts) => sample::sample(opts),
        Subcommand::Pretty => pretty::pretty(),
        Subcommand::Limit(opts) => limit::limit(opts),
    };
    match result {
        Ok(()) => (),
        Err(e) => {
            // Ignore EPIPE
            if let Some(e) = e.downcast_ref::<std::io::Error>() {
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    return ();
                }
            }
            eprintln!(
                "{}: Error: {}",
                std::env::args().collect::<Vec<_>>().join(" "),
                e
            );
            std::process::exit(1)
        }
    }
}
