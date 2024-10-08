mod analyze;
mod label;
mod plot;
mod pretty;
mod sample;

use bpaf::Bpaf;

/// Tools for comparative benchmarking
#[derive(Bpaf)]
#[bpaf(options, fallback_to_usage)]
enum Subcommand {
    Sample(#[bpaf(external(sample::options))] sample::Options),
    Analyze(#[bpaf(external(analyze::options))] analyze::Options),
    Plot(#[bpaf(external(plot::options))] plot::Options),
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();
    let result = match subcommand().run() {
        Subcommand::Sample(opts) => sample::sample(opts),
        Subcommand::Analyze(opts) => analyze::analyze(opts),
        Subcommand::Plot(opts) => plot::plot(opts),
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
