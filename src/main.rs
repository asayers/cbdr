mod run;
mod diff;
mod limit;
mod pretty;
mod label;
mod summarize;

use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    Diff(diff::Options),
    Pretty,
    Limit(limit::Options),
    Summarize,
    Run(run::Options),
}

fn main() {
    env_logger::init();
    let result = match Subcommand::from_args() {
        Subcommand::Diff(opts) => diff::diff(opts),
        Subcommand::Pretty => pretty::pretty(),
        Subcommand::Limit(opts) => limit::limit(opts),
        Subcommand::Summarize => summarize::summarize(),
        Subcommand::Run(opts) => run::all_the_things(opts),
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
