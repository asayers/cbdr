use crate::diff;
use crate::limit;
use crate::pretty;
use crate::sample;
use crate::summarize;
use anyhow::*;
use structopt::*;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(flatten)]
    sample: sample::Options,
    #[structopt(flatten)]
    summarize: summarize::Options,
}

// The cbdr pipeline goes:
//
// sample -> summarize -> diff -> limit -> pretty
//
// This subcommand just runs it all in one process
pub fn all_the_things(opts: Options) -> Result<()> {
    let mut diff = diff::State::new(opts.sample.labels.pairs());
    let (samples, _) = sample::Samples::new(opts.sample)?;
    let mut summarize = summarize::State::new(opts.summarize);
    let mut pretty = pretty::State::new()?;
    for x in samples {
        let (label, values) = x?;
        summarize.update(label, values.into_iter());
        diff.update(&summarize.all_measurements);
        pretty.print(&diff.diffs)?;
    }
    Ok(())
}
