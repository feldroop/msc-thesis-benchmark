use anyhow::Result;

use charming::{
    component::{Axis, Grid, Legend},
    element::{AxisLabel, AxisType, Formatter, Label, LabelPosition, NameLocation, TextStyle},
    series::Bar,
    Chart,
};

use super::{save_chart, JS_FLOAT_FORMATTER, JS_FLOAT_FORMATTER_0};
use crate::{benchmarks::BenchmarkResult, config::BenchmarkSuiteConfig};

use std::iter::zip;

const AXIS_TEXT_SIZE: i32 = 30;

pub fn plot_query_lengths(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let res = benchmark_result.floxer_results.first().unwrap();

    let histogram = &res.stats.query_lengths;

    let thresholds: Vec<_> = histogram
        .thresholds
        .iter()
        .take_while(|&&t| t < 100_000)
        .copied()
        .collect();
    let values = histogram.occurrences_as_i32()[..thresholds.len()].to_vec();
    let mut x_names = histogram.axis_names();
    x_names.resize_with(thresholds.len(), String::new);

    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .data(x_names)
                .axis_label(AxisLabel::new().font_size(AXIS_TEXT_SIZE)),
        )
        .y_axis(Axis::new().axis_label(AxisLabel::new().font_size(AXIS_TEXT_SIZE)))
        .series(Bar::new().data(values));

    save_chart(
        chart,
        String::from("thesis_query_lengths_real"),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_avg_num_anchors_per_seed_and_seed_lengths(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let kept_anchors: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| {
            res.stats
                .anchor_stats_per_seed
                .kept_anchors_per_kept_seed
                .descriptive_stats
                .as_ref()
                .unwrap()
                .mean
        })
        .collect();

    let seed_lengths: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| {
            res.stats
                .seed_stats
                .seed_lengths
                .descriptive_stats
                .as_ref()
                .unwrap()
                .mean
        })
        .collect();

    let instance_names: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.benchmark_instance_name.replace("_", " "))
        .collect();

    let offset_str = "54%";

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right(offset_str).top("10%"))
        .grid(Grid::new().left(offset_str).top("10%"))
        .x_axis(Axis::new().grid_index(0).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("Avg. seed length")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(1).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("Avg. #kept anchors per seed")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(1),
        );

    for (name, (seed_length, kept_anchors)) in zip(instance_names, zip(seed_lengths, kept_anchors))
    {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![seed_length])
                    .name(name.clone())
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![kept_anchors])
                    .name(name)
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            );
    }

    save_chart(
        chart,
        format!("thesis_{}_avg_num_anchors", benchmark_result.benchmark_name),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_num_mapped_and_avg_num_anchors(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let kept_anchors: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| {
            res.stats
                .anchor_stats_per_seed
                .kept_anchors_per_kept_seed
                .descriptive_stats
                .as_ref()
                .unwrap()
                .mean
        })
        .collect();

    let num_mapped: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.mapped_read_stats.num_mapped)
        .collect();

    let instance_names: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.benchmark_instance_name.replace("_", " "))
        .collect();

    let offset_str = "54%";

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right(offset_str).top("10%"))
        .grid(Grid::new().left(offset_str).top("10%"))
        .x_axis(Axis::new().grid_index(0).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("#mapped reads")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(1).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("Avg. #kept anchors per seed")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(1),
        );

    for (name, (num_mapped, kept_anchors)) in zip(instance_names, zip(num_mapped, kept_anchors)) {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![num_mapped])
                    .name(name.clone())
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![kept_anchors])
                    .name(name)
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            );
    }

    save_chart(
        chart,
        format!(
            "thesis_{}_avg_num_anchors_and_num_mapped",
            benchmark_result.benchmark_name
        ),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_profiles(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    // data from profiles was extracted by hand

    // simulated
    // let search_times = vec![
    //     0.9261 * 9166.5,
    //     0.7863 * 4083.1,
    //     0.6644 * 1953.6,
    //     0.0215 * 10876.6,
    // ];
    // let locate_times = vec![
    //     0.0 * 9166.5,
    //     0.0 * 4083.1,
    //     0.0047 * 1953.6,
    //     0.1093 * 10876.6,
    // ];
    // let verify_times = vec![
    //     0.0581 * 9166.5,
    //     0.1854 * 4083.1,
    //     0.2771 * 1953.6,
    //     0.7859 * 10876.6,
    // ];
    // let other_times = vec![
    //     0.0150 * 9166.5,
    //     0.0300 * 4083.1,
    //     0.0600 * 1953.6,
    //     0.1000 * 10876.6,
    // ];

    // real reduced
    // let search_times = vec![
    //     0.8535 * 19806.9,
    //     0.5261 * 5226.6,
    //     0.1160 * 3493.6,
    //     0.0087 * 3961.8,
    // ];
    // let locate_times = vec![
    //     0.0 * 19806.9,
    //     0.0031 * 5226.6,
    //     0.0081 * 3493.6,
    //     0.0409 * 3961.8,
    // ];
    // let verify_times = vec![
    //     0.0968 * 19806.9,
    //     0.4091 * 5226.6,
    //     0.8359 * 3493.6,
    //     0.8352 * 3961.8,
    // ];
    // let other_times = vec![
    //     0.0497 * 19806.9,
    //     0.0617 * 5226.6,
    //     0.0400 * 3493.6,
    //     0.1152 * 3961.8,
    // ];

    // real full
    let search_times = vec![
        0.6461 * 21567.7,
        0.2249 * 7255.7,
        0.0288 * 6016.3,
        0.0240 * 6306.2,
    ];
    let locate_times = vec![
        0.0 * 21567.7,
        0.0013 * 7255.7,
        0.0081 * 6016.3,
        0.0099 * 6306.2,
    ];
    let verify_times = vec![
        0.2829 * 21567.7,
        0.7444 * 7255.7,
        0.9412 * 6016.3,
        0.9651 * 6306.2,
    ];
    let other_times = vec![
        0.0710 * 21567.7,
        0.0307 * 7255.7,
        0.0300 * 6016.3,
        0.0158 * 6306.2,
    ];

    let names = vec!["search", "locate", "verify", "other"];

    let times = vec![search_times, locate_times, verify_times, other_times];

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .x_axis(
            Axis::new()
                .grid_index(0)
                .type_(AxisType::Value)
                .name("CPU Seconds")
                .name_location(NameLocation::Center)
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .axis_label(AxisLabel::new().font_size(20).color("black")),
        )
        .y_axis(
            Axis::new()
                .name("seed errors")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .data(vec!["3", "2", "1", "0"])
                .type_(AxisType::Category),
        );

    for (name, data) in zip(names, times) {
        chart = chart.series(Bar::new().name(name).data(data).stack("true"));
    }

    save_chart(
        chart,
        format!(
            "thesis_{}_profile_comparison",
            benchmark_result.benchmark_name
        ),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_for_soft_anchor_cap(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right("69%").top("15%"))
        .grid(Grid::new().left("36%").right("36%").top("15%"))
        .grid(Grid::new().left("69%").top("15%"))
        .x_axis(Axis::new().grid_index(0).data(vec![""]))
        .y_axis(
            Axis::new()
                .grid_index(0)
                .name("CPU time in seconds")
                .name_text_style(TextStyle::new().font_size(20).color("black"))
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(1).data(vec![""]))
        .y_axis(
            Axis::new()
                .grid_index(1)
                .name("Avg #kept anchors per seed")
                .name_text_style(TextStyle::new().font_size(20).color("black"))
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(2).data(vec![""]))
        .y_axis(
            Axis::new()
                .grid_index(2)
                .name("#mapped reads in 1000")
                .name_text_style(TextStyle::new().font_size(20).color("black"))
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        );

    for res in &benchmark_result.floxer_results {
        chart = chart
            .series(
                Bar::new()
                    .name(res.benchmark_instance_name.replace("_", " "))
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .data(vec![res.resource_metrics.user_cpu_seconds])
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                    ),
            )
            .series(
                Bar::new()
                    .name(res.benchmark_instance_name.replace("_", " "))
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .data(vec![
                        res.stats
                            .anchor_stats_per_seed
                            .kept_anchors_per_kept_seed
                            .descriptive_stats
                            .as_ref()
                            .unwrap()
                            .mean,
                    ])
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                    ),
            )
            .series(
                Bar::new()
                    .name(res.benchmark_instance_name.replace("_", " "))
                    .x_axis_index(2)
                    .y_axis_index(2)
                    .data(vec![res.mapped_read_stats.num_mapped as f64 / 1000.0])
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            );
    }

    save_chart(
        chart,
        "soft_anchor_cap".into(),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_for_soft_anchor_cap_real(
    benchmark_result_0: &BenchmarkResult,
    benchmark_result_1: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let soft_anchor_cap_values = vec!["5", "10", "20", "50", "100"];

    let cpu_times_0 = benchmark_result_0
        .floxer_results
        .iter()
        .map(|res| res.resource_metrics.user_cpu_seconds)
        .collect();

    let cpu_times_1 = benchmark_result_1
        .floxer_results
        .iter()
        .map(|res| res.resource_metrics.user_cpu_seconds)
        .collect();

    let num_mapped_0 = benchmark_result_0
        .floxer_results
        .iter()
        .map(|res| res.mapped_read_stats.num_mapped)
        .collect();

    let num_mapped_1 = benchmark_result_1
        .floxer_results
        .iter()
        .map(|res| res.mapped_read_stats.num_mapped)
        .collect();

    let chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right("55%").top("15%"))
        .grid(Grid::new().left("55%").top("15%"))
        .x_axis(
            Axis::new()
                .grid_index(0)
                .data(soft_anchor_cap_values.clone())
                .name("Soft Anchor Cap")
                .name_location(NameLocation::Center),
        )
        .y_axis(
            Axis::new()
                .grid_index(0)
                .name("CPU Time in Seconds")
                .name_text_style(TextStyle::new().font_size(20).color("black"))
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(
            Axis::new()
                .grid_index(1)
                .data(soft_anchor_cap_values)
                .name("Soft Anchor Cap")
                .name_location(NameLocation::Center),
        )
        .y_axis(
            Axis::new()
                .grid_index(1)
                .name("#Mapped Reads")
                .name_text_style(TextStyle::new().font_size(20).color("black"))
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .series(
            Bar::new()
                .name("0 seed errors")
                .x_axis_index(0)
                .y_axis_index(0)
                .data(cpu_times_0)
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Top)
                        .font_size(18)
                        .color("black")
                        .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                ),
        )
        .series(
            Bar::new()
                .name("1 seed errors")
                .x_axis_index(0)
                .y_axis_index(0)
                .data(cpu_times_1)
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Top)
                        .font_size(18)
                        .color("black")
                        .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                ),
        )
        .series(
            Bar::new()
                .name("0 seed errors")
                .x_axis_index(1)
                .y_axis_index(1)
                .data(num_mapped_0)
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Top)
                        .font_size(18)
                        .color("black")
                        .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                ),
        )
        .series(
            Bar::new()
                .name("1 seed errors")
                .x_axis_index(1)
                .y_axis_index(1)
                .data(num_mapped_1)
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Top)
                        .font_size(18)
                        .color("black")
                        .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                ),
        );

    save_chart(
        chart,
        "soft_anchor_cap_real".into(),
        1200,
        800,
        &benchmark_result_0.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_cpu_times_and_num_mapped(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let cpu_times: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.resource_metrics.user_cpu_seconds)
        .collect();

    let num_mapped: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.mapped_read_stats.num_mapped)
        .collect();

    let instance_names: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.benchmark_instance_name.replace("_", " "))
        .collect();

    let offset_str = "54%";

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right(offset_str).top("10%"))
        .grid(Grid::new().left(offset_str).top("10%"))
        .x_axis(Axis::new().grid_index(0).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("CPU Seconds")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(1).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("#Mapped Reads")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(1),
        );

    for (name, (cpu_time, num_mapped)) in zip(instance_names, zip(cpu_times, num_mapped)) {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![cpu_time])
                    .name(name.clone())
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![num_mapped])
                    .name(name)
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                    ),
            );
    }

    save_chart(
        chart,
        format!(
            "{}_cpu_times_and_num_mapped",
            benchmark_result.benchmark_name
        ),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_cpu_times_and_seed_lengths_and_num_mapped(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let cpu_times: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.resource_metrics.user_cpu_seconds)
        .collect();

    let seed_lengths: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| {
            res.stats
                .seed_stats
                .seed_lengths
                .descriptive_stats
                .as_ref()
                .unwrap()
                .mean
        })
        .collect();

    let num_mapped: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.mapped_read_stats.num_mapped)
        .collect();

    let instance_names: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.benchmark_instance_name.replace("_", " "))
        .collect();

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right("67%").top("10%"))
        .grid(Grid::new().left("38%").right("38%").top("10%"))
        .grid(Grid::new().left("67%").top("10%"))
        .x_axis(Axis::new().grid_index(0).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("CPU Seconds")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(1).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("Avg. Seed Lengths")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(1),
        )
        .x_axis(Axis::new().grid_index(2).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("#Mapped Reads")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(2),
        );

    for (name, (cpu_time, (seed_length, num_mapped))) in zip(
        instance_names,
        zip(cpu_times, zip(seed_lengths, num_mapped)),
    ) {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![cpu_time])
                    .name(name.clone())
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![seed_length])
                    .name(name.clone())
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![num_mapped])
                    .name(name)
                    .x_axis_index(2)
                    .y_axis_index(2)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                    ),
            );
    }

    save_chart(
        chart,
        format!(
            "{}_cpu_times_and_seed_lengths_and_num_mapped",
            benchmark_result.benchmark_name
        ),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_seed_errors_and_num_mapped_and_num_seeds(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let seed_errors: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| {
            res.stats
                .seed_stats
                .errors_per_seed
                .descriptive_stats
                .as_ref()
                .unwrap()
                .mean
        })
        .collect();

    let num_seeds: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| {
            res.stats
                .seed_stats
                .seeds_per_query
                .descriptive_stats
                .as_ref()
                .unwrap()
                .mean
        })
        .collect();

    let num_mapped: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.mapped_read_stats.num_mapped)
        .collect();

    let instance_names: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.benchmark_instance_name.replace("_", " "))
        .collect();

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right("67%").top("10%"))
        .grid(Grid::new().left("38%").right("38%").top("10%"))
        .grid(Grid::new().left("67%").top("10%"))
        .x_axis(Axis::new().grid_index(0).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("Avg. #Seed Length")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(1).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("Avg. #Seeds Per Read")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(1)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(2).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("#Mapped Reads")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(2),
        );

    for (name, (seed_errors, (num_seeds, num_mapped))) in
        zip(instance_names, zip(seed_errors, zip(num_seeds, num_mapped)))
    {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![seed_errors])
                    .name(name.clone())
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![num_seeds])
                    .name(name.clone())
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![num_mapped])
                    .name(name)
                    .x_axis_index(2)
                    .y_axis_index(2)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                    ),
            );
    }

    save_chart(
        chart,
        format!(
            "{}_num_errors_num_seeds_and_num_mapped",
            benchmark_result.benchmark_name
        ),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_two_different_cpu_times(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let cpu_times: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.resource_metrics.user_cpu_seconds)
        .collect();

    let full_output_cpu_times = vec![8904.59, 8182.83, 7755.14, 7872.75, 7935.83];

    let mut instance_names: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.benchmark_instance_name.replace("_", "."))
        .collect();

    instance_names
        .last_mut()
        .unwrap()
        .push_str(" extra verification ratio");

    let offset_str = "54%";

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right(offset_str).top("10%"))
        .grid(Grid::new().left(offset_str).top("10%"))
        .x_axis(
            Axis::new()
                .grid_index(0)
                .data(vec!["Reduced Output Mode"])
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .y_axis(
            Axis::new()
                .name("CPU Seconds")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(
            Axis::new()
                .grid_index(1)
                .data(vec!["Full Output Mode"])
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .y_axis(
            Axis::new()
                .name("CPU Seconds")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(1),
        );

    for (name, (cpu_time, full_output_cpu_time)) in
        zip(instance_names, zip(cpu_times, full_output_cpu_times))
    {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![cpu_time])
                    .name(name.clone())
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![full_output_cpu_time])
                    .name(name)
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            );
    }

    save_chart(
        chart,
        format!("{}_cpu_times", benchmark_result.benchmark_name),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_cpu_times_and_num_root_alignments_and_num_mapped(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let cpu_times: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.resource_metrics.user_cpu_seconds)
        .collect();

    let num_root_alignments: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| {
            res.stats
                .alignment_stats
                .reference_span_sizes_aligned_of_roots
                .num_values as i64
                / 1000
        })
        .collect();

    let num_mapped: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.mapped_read_stats.num_mapped)
        .collect();

    let instance_names: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.benchmark_instance_name.replace("_", " "))
        .collect();

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right("67%").top("10%"))
        .grid(Grid::new().left("38%").right("38%").top("10%"))
        .grid(Grid::new().left("67%").top("10%"))
        .x_axis(Axis::new().grid_index(0).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("CPU Seconds")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(1).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("#Long-Read Alignments in 1000")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(1),
        )
        .x_axis(Axis::new().grid_index(2).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("#Mapped Reads")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(2),
        );

    for (name, (cpu_time, (num_root_alignments, num_mapped))) in zip(
        instance_names,
        zip(cpu_times, zip(num_root_alignments, num_mapped)),
    ) {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![cpu_time])
                    .name(name.clone())
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![num_root_alignments])
                    .name(name.clone())
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![num_mapped])
                    .name(name)
                    .x_axis_index(2)
                    .y_axis_index(2)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                    ),
            );
    }

    save_chart(
        chart,
        format!(
            "{}_cpu_times_and_num_alignments_and_num_mapped",
            benchmark_result.benchmark_name
        ),
        1400,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}

pub fn plot_avg_anchors_per_query_and_num_mapped(
    benchmark_result: &BenchmarkResult,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let avg_seeds_per_query: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| {
            res.stats
                .anchor_stats_per_query
                .kept_anchors_per_query
                .descriptive_stats
                .as_ref()
                .unwrap()
                .mean
        })
        .collect();

    let num_mapped: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.mapped_read_stats.num_mapped)
        .collect();

    let mut instance_names: Vec<_> = benchmark_result
        .floxer_results
        .iter()
        .map(|res| res.benchmark_instance_name.replace("_", " "))
        .collect();

    instance_names
        .last_mut()
        .unwrap()
        .push_str(" seed sampling step size");

    let offset_str = "54%";

    let mut chart = Chart::new()
        .background_color("white")
        .legend(Legend::new().text_style(TextStyle::new().font_size(20).color("black")))
        .grid(Grid::new().right(offset_str).top("10%"))
        .grid(Grid::new().left(offset_str).top("10%"))
        .x_axis(Axis::new().grid_index(0).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("        Avg. #Anchors Per Read")
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .x_axis(Axis::new().grid_index(1).data(vec![""]))
        .y_axis(
            Axis::new()
                .name("#Mapped Reads")
                .axis_label(AxisLabel::new().font_size(18).color("black"))
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .grid_index(1),
        );

    for (name, (avg_seeds_per_query, num_mapped)) in
        zip(instance_names, zip(avg_seeds_per_query, num_mapped))
    {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![avg_seeds_per_query])
                    .name(name.clone())
                    .x_axis_index(0)
                    .y_axis_index(0)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                    ),
            )
            .series(
                Bar::new()
                    .data(vec![num_mapped])
                    .name(name)
                    .x_axis_index(1)
                    .y_axis_index(1)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Top)
                            .font_size(18)
                            .color("black")
                            .formatter(Formatter::Function(JS_FLOAT_FORMATTER_0.into())),
                    ),
            );
    }

    save_chart(
        chart,
        format!(
            "{}_avg_anchors_and_num_mapped",
            benchmark_result.benchmark_name
        ),
        1200,
        800,
        &benchmark_result.folder,
        suite_config,
    );

    Ok(())
}
