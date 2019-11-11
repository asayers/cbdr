use rolling_stats::Stats;
use std::io::{BufWriter, Write};

fn main() {
    match main2() {
        Ok(()) | Err(Error::EPipe) => (),
        Err(_) => std::process::exit(1),
    }
}

fn main2() -> Result<(), Error> {
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let mut hdrs = rdr.headers()?.into_iter();
    let key_label = hdrs.next().unwrap().to_string();
    writeln!(stdout, "{},field,count,mean,stddev,min,max", key_label)?;
    eprintln!("Grouping by {}", key_label);
    let labels = hdrs.map(str::to_string).collect::<Vec<String>>();
    let mut statss: Vec<Stats<f64>> = vec![Stats::new(); labels.len()];
    let mut count = 0;
    let mut cur_key: Option<String> = None;

    for row in rdr.into_records() {
        let row = row?;
        let mut fields = row.into_iter();
        let key = fields.next().unwrap();
        if cur_key.is_none() {
            cur_key = Some(key.into());
        }
        if cur_key.as_ref().unwrap() != key {
            for (label, stats) in labels.iter().zip(&mut statss) {
                writeln!(
                    stdout,
                    "{},{},{},{},{},{},{}",
                    cur_key.as_ref().unwrap(),
                    label,
                    count,
                    stats.mean,
                    stats.std_dev,
                    stats.min,
                    stats.max
                )?;
                *stats = Stats::new();
            }
            count = 0;
            cur_key = Some(key.to_string());
        }
        count += 1;
        for (x, stats) in fields.zip(&mut statss) {
            stats.update(x.parse::<f64>()?);
        }
    }
    for (label, stats) in labels.iter().zip(&mut statss) {
        writeln!(
            stdout,
            "{},{},{},{},{},{},{}",
            cur_key.as_ref().unwrap(),
            label,
            count,
            stats.mean,
            stats.std_dev,
            stats.min,
            stats.max
        )?;
    }
    Ok(())
}

#[derive(Debug)]
enum Error {
    EPipe,
    Other(Box<dyn std::error::Error>),
}
macro_rules! error_impl {
    ($t:ty) => {
        impl From<$t> for Error {
            fn from(x: $t) -> Error {
                Error::Other(Box::new(x))
            }
        }
    };
}
error_impl!(std::num::ParseFloatError);
error_impl!(csv::Error);
impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Error {
        if x.kind() == std::io::ErrorKind::BrokenPipe {
            Error::EPipe
        } else {
            Error::Other(Box::new(x))
        }
    }
}
