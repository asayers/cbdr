use crate::label::*;
use anyhow::*;
use log::*;
use serde_json::json;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    about = "Takes CSV data on stdin and produces a vega-lite plot specification on stdout"
)]
pub struct Options {}

pub fn plot(opts: Options) -> Result<()> {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let mut headers = rdr.headers().unwrap().into_iter();
    let benchcol = headers.next().unwrap().to_string();
    info!("Assuming \"{}\" column is the benchmark name", benchcol);
    init_metrics(headers.map(|x| x.to_string()).collect());

    let data = rdr
        .into_records()
        .map(|row| {
            let row = row.unwrap();
            let mut row = row.into_iter();
            let bench = Bench::from(row.next().unwrap());
            let mut map = serde_json::Map::<String, serde_json::Value>::new();
            map.insert(benchcol.clone(), json!(bench));
            for (x, y) in all_metrics().zip(row) {
                map.insert(x.to_string(), json!(y.parse::<f64>().unwrap()));
            }
            map
        })
        .collect::<Vec<_>>();
    let mk_chart = |metric: Metric| {
        json!({
            "title": metric,
            "width": 640,
            "height": 180,
            "mark": {
                "type": "area",
                "opacity": 0.5,
            },
            "encoding": {
                "x": { "field": "value", "type": "quantitative" },
                "y": { "field": "density", "type": "quantitative" },
                "color": { "field": "benchmark", "type": "nominal" },
            },
            "transform": [{
                "density": metric,
                "groupby": ["benchmark"],
            }],
        })
    };
    let mut charts = all_metrics().map(mk_chart).collect::<Vec<_>>();
    charts.reverse();
    let plot = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v4.json",
        "vconcat": charts,
        "data": { "values": data },
    });

    println!("{}", plot);
    Ok(())
}
