use ansi_term::{Color, Style};
use confidence::*;
use log::*;
use std::collections::HashMap;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    threshold: Option<f64>,
    #[structopt(short, long, default_value = "0.95")]
    significance_level: f64,
    comparisons: Vec<String>,
    #[structopt(short, long)]
    csv: bool,
}

fn main() {
    env_logger::init();
    let opts = Options::from_args();
    main2(opts).unwrap();
}

fn main2(opts: Options) -> Result<(), Box<dyn std::error::Error>> {
    let comparisons = opts
        .comparisons
        .iter()
        .flat_map(|x| {
            x.split(',')
                .zip(x.split(',').skip(1))
                .map(|(from, to)| (from.into(), to.into()))
        })
        .collect::<Vec<(String, String)>>();

    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let stat_names = rdr
        .headers()
        .unwrap()
        .into_iter()
        .skip(1)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let mut state = State::new(comparisons, stat_names, opts.significance_level);
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let label = row.next().unwrap().to_string();
        state.update_measurements(&label, row.map(|x| x.parse().unwrap()));
        if log_enabled!(log::Level::Info) {
            // state.print_status();
            eprintln!("----------");
            state.print_pretty(std::io::stderr())?;
        }

        if let Some(t) = opts.threshold {
            if state.is_finished(t) {
                break;
            }
        }
    }
    let stdout = std::io::stdout();
    if opts.csv {
        state.print_csv(stdout)?;
    } else {
        state.print_pretty(stdout)?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Measurements {
    count: usize,
    stats: Vec<rolling_stats::Stats<f64>>,
}
impl Measurements {
    fn new(n: usize) -> Measurements {
        Measurements {
            count: 0,
            stats: vec![rolling_stats::Stats::new(); n],
        }
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = Stats> + 'a {
        self.stats.iter().map(move |x| Stats {
            count: self.count,
            mean: x.mean,
            std_dev: x.std_dev,
        })
    }
}

struct State {
    significance_level: f64,
    comparisons: Vec<(String, String)>,
    stat_names: Vec<String>,
    measurements: HashMap<String, Measurements>,
    // Outer vec corresponds to comparison, inner to stat_name
    cis: Vec<Vec<Option<ConfidenceInterval>>>,
}

impl State {
    fn new(
        comparisons: Vec<(String, String)>,
        stat_names: Vec<String>,
        significance_level: f64,
    ) -> State {
        let mut cis = vec![];
        for _ in 0..comparisons.len() {
            cis.push(vec![None; stat_names.len()]);
        }
        State {
            significance_level,
            measurements: HashMap::new(),
            cis,
            comparisons,
            stat_names,
        }
    }

    fn update_measurements(&mut self, label: &str, values: impl Iterator<Item = f64>) {
        let n = self.stat_names.len();
        let entry = self
            .measurements
            .entry(label.to_string())
            .or_insert_with(|| Measurements::new(n));
        entry.count += 1;
        for (stats, value) in entry.stats.iter_mut().zip(values) {
            stats.update(value);
        }
    }

    fn update_cis(&mut self) {
        let sig_level = self.significance_level;
        for (i, (from, to)) in self.comparisons.iter_mut().enumerate() {
            if let Some(from) = self.measurements.get(from) {
                if let Some(to) = self.measurements.get(to) {
                    self.cis[i].clear();
                    self.cis[i].extend(
                        from.iter()
                            .zip(to.iter())
                            .map(|(x, y)| confidence_interval(sig_level, x, y)),
                    );
                }
            }
        }
    }

    // // TODO: configurable logging
    // fn print_status(&mut self) {
    //     use std::fmt::Write;
    //     self.update_cis();
    //     let num_measurements = self
    //         .measurements
    //         .iter()
    //         .map(|(_, x)| x.count)
    //         .collect::<Vec<_>>();
    //     let mut buf = String::new();
    //     for cis in &self.cis {
    //         for ci in cis {
    //             write!(buf, "\t{}", PrettyCI(*ci)).unwrap();
    //         }
    //     }
    //     info!("{:03?} {}", num_measurements, buf);
    // }

    fn is_finished(&self, threshold: f64) -> bool {
        self.cis.iter().all(|cis| {
            cis.iter()
                .all(|ci| ci.as_ref().map_or(false, |ci| ci.radius < threshold))
        })
    }

    fn print_pretty(&mut self, stdout: impl Write) -> Result<(), Box<std::io::Error>> {
        self.update_cis();
        let mut stdout = tabwriter::TabWriter::new(stdout);
        write!(stdout, "from\t\tto\t")?;
        for x in &self.stat_names {
            write!(stdout, " {:^21}", x)?;
        }
        writeln!(stdout,)?;
        for (comp, cis) in self.comparisons.iter().zip(self.cis.iter()) {
            write!(stdout, "{}\t..\t{}\t", comp.0, comp.1)?;
            for ci in cis {
                write!(stdout, " {}", PrettyCI(*ci))?;
            }
            writeln!(stdout,)?;
        }
        stdout.flush()?;
        Ok(())
    }

    fn print_csv(&mut self, mut stdout: impl Write) -> Result<(), Box<std::io::Error>> {
        self.update_cis();
        write!(stdout, "from,to")?;
        for x in &self.stat_names {
            write!(stdout, ",{}", x)?;
        }
        writeln!(stdout)?;
        for (comp, cis) in self.comparisons.iter().zip(self.cis.iter()) {
            write!(stdout, "{},{}", comp.0, comp.1)?;
            for ci in cis {
                if let Some(ci) = ci {
                    write!(stdout, ",{}", ci)?;
                } else {
                    write!(stdout, ",insufficient data")?;
                }
            }
            writeln!(stdout)?;
        }
        Ok(())
    }
}

use std::fmt;
// Always takes 21 characters
struct PrettyCI(Option<ConfidenceInterval>);
impl fmt::Display for PrettyCI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ci) = self.0 {
            if ci.center - ci.radius < 0. && 0. < ci.center + ci.radius {
                let center = format!("{:.3e}", ci.center);
                let radius = format!("{:.3e}", ci.radius);
                write!(f, "{:>9} ± {:<9}", center, radius)
            } else {
                let center = format!("{:.3e}", ci.center);
                let radius = format!("{:.3e}", ci.radius);
                write!(
                    f,
                    "{}{:>9} ± {:<9}{}",
                    Color::Yellow.prefix(),
                    center,
                    radius,
                    Color::Yellow.suffix()
                )
            }
        } else {
            write!(f, "{}", Style::new().dimmed().paint("  insufficient data "))
        }
    }
}
