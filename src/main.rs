mod diff;
mod pretty;
mod sample;

use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    Diff(diff::Options),
    Sample(sample::Options),
    Pretty,
}

fn main() {
    env_logger::init();
    let result = match Subcommand::from_args() {
        Subcommand::Diff(opts) => diff::diff(opts),
        Subcommand::Sample(opts) => sample::sample(opts),
        Subcommand::Pretty => pretty::pretty(),
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
            eprintln!("Error: {}", e);
            std::process::exit(1)
        }
    }
}
