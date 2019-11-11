fn main() {
    println!("Hello, world!");
}

fn run(commit_a: Commit, commit_b: Commit) -> ConfidenceInterval {
    let all_measurements = Set::<Measurement>::new();
    loop {
        let measurement = bench(if rand().is_even() { commit_a } else { commit_b });
        all_measurements.insert(measurement);
        let ci = confidence_interval(all_measurements);
        if ci.width() < THRESHOLD { return ci; }
    }
}
