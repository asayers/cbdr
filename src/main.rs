mod analyze;
mod label;
mod pretty;
mod sample;
mod summarize;

use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    Sample(sample::Options),
    Analyze(analyze::Options),
}

fn main() {
    env_logger::init();
    let result = match Subcommand::from_args() {
        Subcommand::Sample(opts) => sample::sample(opts),
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
            eprint!("Error");
            for e in e.chain() {
                eprint!(": {}", e);
            }
            eprintln!();
            std::process::exit(1)
        }
    }
}
