use std::io::Write;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    benches: Vec<String>,
}

fn main() {
    match main2() {
        Ok(()) => (),
        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => (),
        _ => std::process::exit(1),
    }
}
fn main2() -> Result<(), Box<std::io::Error>> {
    let opts = Options::from_args();
    for (idx, bench) in opts.benches.iter().enumerate() {
        eprintln!("Warming up {}...", idx);
        Command::new("/bin/sh")
            .arg("-c")
            .arg(bench)
            .output()
            .unwrap();
    }

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    writeln!(stdout, "idx,measurement")?;
    loop {
        let idx = rand::random::<usize>() % opts.benches.len();
        let out = Command::new("/bin/sh")
            .arg("-c")
            .arg(&opts.benches[idx])
            .output()
            .unwrap()
            .stdout;
        write!(stdout, "{},", idx)?;
        stdout.write_all(&out)?;
    }
}
