use crate::label::*;
use anyhow::Result;
use bpaf::Bpaf;
use log::*;
use serde_json::json;

/// Takes CSV data on stdin and produces a vega-lite plot specification on stdout
#[derive(Bpaf)]
pub struct Options {
    omit_data: bool,
}

pub fn mk_chart(metric: Metric) -> serde_json::Value {
    let metric = metric.to_string();

    // It's a bit hacky, but we special-case any metrics with these well-known
    // names and optimize their chart.
    if metric == "user_time" || metric == "sys_time" {
        json!({
            "title": metric,
            "width": 640,
            "height": 180,
            "mark": {
                "type": "area",
                "interpolate": "monotone",
                "opacity": 0.5,
            },
            "encoding": {
                "x": { "field": metric, "type": "ordinal" },
                "y": { "aggregate": "count", "type": "quantitative", "stack":null },
                "color": { "field": "benchmark", "type": "nominal" },
            },
        })
    } else {
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
            "transform": [
                {
                    "density": metric,
                    "steps": 1000,
                    "groupby": ["benchmark"],
                },
                { "filter": "datum.density > 1" },
            ],
        })
    }
}

pub fn plot(opts: Options) -> Result<()> {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let mut headers = rdr.headers().unwrap().into_iter();
    let benchcol = headers.next().unwrap().to_string();
    info!("Assuming \"{}\" column is the benchmark name", benchcol);
    init_metrics(headers.map(|x| x.to_string()).collect());

    let mut charts = all_metrics().map(mk_chart).collect::<Vec<_>>();
    charts.reverse();

    let plot = if opts.omit_data {
        json!({
            "$schema": "https://vega.github.io/schema/vega-lite/v4.json",
            "vconcat": charts,
        })
    } else {
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
        json!({
            "$schema": "https://vega.github.io/schema/vega-lite/v4.json",
            "data": { "values": data },
            "vconcat": charts,
        })
    };

    println!("{}", plot);
    Ok(())
}
