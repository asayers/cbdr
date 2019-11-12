use ansi_term::{Color, Style};
use confidence::*;
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    threshold: Option<f64>,
    #[structopt(short, long, default_value = "0.95")]
    significance_level: f64,
    comparisons: Vec<String>,
}

fn main() {
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
    let mut state = State::new(comparisons, stat_names);
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let label = row.next().unwrap().to_string();
        state.update_measurements(&label, row.map(|x| x.parse().unwrap()));
        state.update_cis(opts.significance_level);
        state.print_status();

        if let Some(t) = opts.threshold {
            if state.is_finished(t) {
                break;
            }
        }
    }
    state.finalize();
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
    comparisons: Vec<(String, String)>,
    stat_names: Vec<String>,
    measurements: HashMap<String, Measurements>,
    // Outer vec corresponds to comparison, inner to stat_name
    cis: Vec<Vec<Option<ConfidenceInterval>>>,
}

impl State {
    fn new(comparisons: Vec<(String, String)>, stat_names: Vec<String>) -> State {
        let mut cis = vec![];
        for _ in 0..comparisons.len() {
            cis.push(vec![None; stat_names.len()]);
        }
        State {
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

    fn update_cis(&mut self, significance_level: f64) {
        for (i, (from, to)) in self.comparisons.iter_mut().enumerate() {
            if let Some(from) = self.measurements.get(from) {
                if let Some(to) = self.measurements.get(to) {
                    self.cis[i].clear();
                    self.cis[i].extend(
                        from.iter()
                            .zip(to.iter())
                            .map(|(x, y)| confidence_interval(significance_level, x, y)),
                    );
                }
            }
        }
    }

    fn print_status(&self) {
        let num_measurements = self
            .measurements
            .iter()
            .map(|(_, x)| x.count)
            .collect::<Vec<_>>();
        eprint!("{:03?}", num_measurements);
        for cis in &self.cis {
            for ci in cis {
                match ci {
                    None => eprint!("\t{}", Style::new().dimmed().paint("insufficient data")),
                    Some(ref ci) if ci.center - ci.radius < 0. && 0. < ci.center + ci.radius => {
                        eprint!("\t{:.6} ± {:.6}", ci.center, ci.radius,)
                    }
                    Some(ci) => eprint!(
                        "\t{}{:.6} ± {:.6}{}",
                        Color::Yellow.prefix(),
                        ci.center,
                        ci.radius,
                        Color::Yellow.suffix()
                    ),
                }
            }
        }
        eprintln!();
    }

    // fn all_cis(&self) -> impl Iterator<Item = Option<ConfidenceInterval>> {
    //     let l = self.stat_names.len();
    //     self.comparisons
    //         .clone()
    //         .into_iter()
    //         .flat_map(|x| (0..l).map(|i| self.cis.get(&(x, i)).cloned()))
    // }

    fn is_finished(&self, threshold: f64) -> bool {
        self.cis.iter().all(|cis| {
            cis.iter()
                .all(|ci| ci.as_ref().map_or(false, |ci| ci.radius < threshold))
        })
    }

    fn finalize(self) {
        print!("from,to");
        for x in self.stat_names {
            print!(",{}", x);
        }
        println!();
        for (comp, cis) in self.comparisons.into_iter().zip(self.cis.into_iter()) {
            print!("{},{}", comp.0, comp.1);
            for ci in cis {
                if let Some(ci) = ci {
                    print!(",{}", ci);
                } else {
                    print!(",insufficient data");
                }
            }
            println!();
        }
    }
}
