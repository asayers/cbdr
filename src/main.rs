mod diff;
mod sample;

use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    Diff(diff::Options),
    Sample(sample::Options),
}

fn main() {
    env_logger::init();
    let result = match Subcommand::from_args() {
        Subcommand::Diff(opts) => diff::diff(opts),
        Subcommand::Sample(opts) => sample::sample(opts),
    };
    match result {
        Ok(()) => (),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<std::io::Error>() {
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    return ();
                }
            }
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
