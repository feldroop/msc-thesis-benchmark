pub mod thesis;

use std::fs;

use crate::{
    analyze_mapped_reads::{DetailedMappedReadsComparison, SimpleMappedReadsStats},
    config::BenchmarkSuiteConfig,
    folder_structure::BenchmarkFolder,
    readmappers::{floxer::HistogramData, ResourceMetrics},
};

use charming::{
    component::{Axis, Grid, Legend, Title},
    element::{AxisLabel, Formatter, Label, LabelPosition, NameLocation, TextStyle},
    series::Bar,
    Chart, ImageRenderer,
};

const AXIS_TEXT_SIZE: i32 = 25;

static SUBTITLE_FONT_SIZE: u8 = 20;
static LABEL_FONT_SIZE: u8 = 15;
static GRID_OUTERMOST_OFFSET: usize = 6;
static JS_FLOAT_FORMATTER: &str = "function (param) { return param.data.toFixed(1); }";
static JS_FLOAT_FORMATTER_0: &str = "function (param) { return param.data.toFixed(0); }";

pub fn plot_resource_metrics<'a>(
    benchmark_name: &str,
    metrics_and_names_of_runs: impl IntoIterator<Item = (&'a ResourceMetrics, &'a str)>,
    benchmark_folder: &BenchmarkFolder,
    suite_config: &BenchmarkSuiteConfig,
) {
    let mut chart = Chart::new()
        .legend(
            Legend::new()
                .right("10%")
                .text_style(TextStyle::new().font_size(20).color("black")),
        )
        .grid(Grid::new().right("37%").top("10%"))
        .grid(Grid::new().left("71%").top("10%"))
        .background_color("white")
        .x_axis(
            Axis::new()
                .data(vec!["Wall Time", "User CPU Time", "System CPU Time"])
                .grid_index(0)
                .name_text_style(TextStyle::new().font_size(AXIS_TEXT_SIZE).color("black"))
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .y_axis(
            Axis::new()
                .name("Seconds")
                .name_text_style(TextStyle::new().font_size(AXIS_TEXT_SIZE).color("black"))
                .grid_index(0)
                .axis_label(AxisLabel::new().font_size(AXIS_TEXT_SIZE).color("black")),
        )
        .x_axis(
            Axis::new()
                .data(vec!["Peak Memory Usage"])
                .grid_index(1)
                .name_text_style(TextStyle::new().font_size(AXIS_TEXT_SIZE).color("black"))
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .y_axis(
            Axis::new()
                .name("Gibibytes")
                .name_text_style(TextStyle::new().font_size(AXIS_TEXT_SIZE).color("black"))
                .grid_index(1)
                .axis_label(AxisLabel::new().font_size(AXIS_TEXT_SIZE).color("black")),
        );

    for (metrics, name) in metrics_and_names_of_runs.into_iter() {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![
                        metrics.wall_clock_seconds,
                        metrics.user_cpu_seconds,
                        metrics.system_cpu_seconds,
                    ])
                    .name(name.replace("_", " "))
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
                    .data(vec![
                        (metrics.peak_memory_kilobytes as i64) / 1_000_000,
                        // metrics.average_memory_kilobytes as i64, <-- seems to be not available and is not as important
                    ])
                    .name(name.replace("_", " "))
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
        format!("{benchmark_name}_resource_metrics"),
        1200,
        800,
        benchmark_folder,
        suite_config,
    );
}

pub fn plot_mapped_reads_stats<'a, S>(
    iter: impl IntoIterator<Item = &'a SimpleMappedReadsStats>,
    title: &str,
    instance_names: impl IntoIterator<Item = S>,
    benchmark_folder: &BenchmarkFolder,
    suite_config: &BenchmarkSuiteConfig,
) where
    S: AsRef<str>,
{
    let instance_names: Vec<_> = instance_names
        .into_iter()
        .map(|s| s.as_ref().to_owned())
        .collect();

    let num_mapped_per_instance: Vec<_> = iter.into_iter().map(|stats| stats.num_mapped).collect();

    let chart = Chart::new()
        .background_color("white")
        .x_axis(
            Axis::new()
                .data(instance_names.clone())
                .axis_label(AxisLabel::new().font_size(AXIS_TEXT_SIZE)),
        )
        .y_axis(
            Axis::new()
                .name("Number of aligned queries")
                .name_text_style(TextStyle::new().font_size(AXIS_TEXT_SIZE))
                .axis_label(AxisLabel::new().font_size(AXIS_TEXT_SIZE)),
        )
        .series(
            Bar::new()
                .data(num_mapped_per_instance)
                .x_axis_index(0)
                .y_axis_index(0)
                .label(Label::new().show(true).position(LabelPosition::Top)),
        );

    let plot_name_for_file = title.to_ascii_lowercase().replace(' ', "_");

    save_chart(
        chart,
        plot_name_for_file,
        1200,
        800,
        benchmark_folder,
        suite_config,
    );
}

pub fn plot_histogram_data_in_grid<'a, I, S1, S2>(
    iter: impl IntoIterator<Item = I>,
    title: &str,
    instance_names: impl IntoIterator<Item = S1>,
    metric_names: impl IntoIterator<Item = S2>,
    benchmark_folder: &BenchmarkFolder,
    suite_config: &BenchmarkSuiteConfig,
) where
    I: IntoIterator<Item = &'a HistogramData>,
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    // runs should be columns, metrics are rows
    let runs: Vec<Vec<&HistogramData>> = iter
        .into_iter()
        .map(|run| run.into_iter().collect())
        .collect();

    let mut y_max_per_metric = Vec::new();
    for metric_index in 0..(runs[0].len()) {
        let y_max = runs
            .iter()
            .map(|run| {
                nice_upper_bound(*run[metric_index].occurrences.iter().max().unwrap()) as i32
            })
            .max()
            .unwrap();
        y_max_per_metric.push(y_max);
    }

    let num_columns = runs.len();
    let num_rows = runs.first().expect("at least one run to plot").len();

    let available_column_space = 105f64;
    let available_row_space = 90f64;
    let column_width = available_column_space / num_columns as f64;
    let row_width = available_row_space / num_rows as f64;

    let subtitle_top_offset_percentage = 4;
    let grid_extra_top_offset_percentage = 3 * subtitle_top_offset_percentage; // to make space for the subtitles

    let instance_names: Vec<_> = instance_names
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect();

    let metric_names: Vec<_> = metric_names
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect();

    let mut chart = Chart::new()
        .legend(
            Legend::new()
                .text_style(TextStyle::new().font_size(SUBTITLE_FONT_SIZE))
                .top("top"),
        )
        .background_color("white");

    for (column_index, metrics) in runs.iter().enumerate() {
        let column_index: usize = column_index; // fix rust-analyzer false-positive error

        let left_offset_str = format!("{}%", (column_index as f64 * column_width) as usize);
        let right_offset_str = format!(
            "{}%",
            ((num_columns - 1 - column_index) as f64 * column_width) as usize
        );

        chart = chart.title(create_title_with_offsets(
            &instance_names[column_index],
            &left_offset_str,
            &format!("{subtitle_top_offset_percentage}%"),
        ));

        for (row_index, histogram) in metrics.iter().enumerate() {
            let row_index: usize = row_index; // fix rust-analyzer false-positive error

            let top_offset_str = format!(
                "{}%",
                grid_extra_top_offset_percentage + (row_index as f64 * row_width) as usize
            );
            let bottom_offset_str = format!(
                "{}%",
                GRID_OUTERMOST_OFFSET + ((num_rows - 1 - row_index) as f64 * row_width) as usize
            );

            chart = chart.grid(create_grid_with_offsets(
                &left_offset_str,
                &right_offset_str,
                &top_offset_str,
                &bottom_offset_str,
            ));

            let index = num_rows * column_index + row_index;
            chart = add_histogram_data_to_chart(
                chart,
                histogram,
                &metric_names[row_index],
                index as i32,
                y_max_per_metric[row_index],
            );
        }
    }

    let plot_name_for_file = title.to_ascii_lowercase().replace(' ', "_");

    save_chart(
        chart,
        plot_name_for_file,
        600 * num_columns as u32,
        400 * num_rows as u32,
        benchmark_folder,
        suite_config,
    );
}

fn create_grid_with_offsets(
    left_offset_str: &str,
    right_offset_str: &str,
    top_offset_str: &str,
    bottom_offset_str: &str,
) -> Grid {
    let default_offset_str = format!("{GRID_OUTERMOST_OFFSET}%");
    let mut grid = Grid::new();

    // setting the offsets to 0% somehow destroys the layout
    grid = grid.left(if left_offset_str != "0%" {
        left_offset_str
    } else {
        &default_offset_str
    });

    grid = grid.right(if right_offset_str != "0%" {
        right_offset_str
    } else {
        &default_offset_str
    });

    grid = grid.top(if top_offset_str != "0%" {
        top_offset_str
    } else {
        &default_offset_str
    });

    grid = grid.bottom(if bottom_offset_str != "0%" {
        bottom_offset_str
    } else {
        &default_offset_str
    });

    grid
}

fn create_title_with_offsets(text: &str, left_offset_str: &str, top_offset_str: &str) -> Title {
    let default_offset_str = "3%";
    let mut title = Title::new()
        .text(text)
        .text_style(TextStyle::new().font_size(SUBTITLE_FONT_SIZE));

    // setting the offsets to 0% somehow destroys the layout
    title = title.left(if left_offset_str != "0%" {
        left_offset_str
    } else {
        default_offset_str
    });

    title = title.top(if top_offset_str != "0%" {
        top_offset_str
    } else {
        default_offset_str
    });

    title
}

fn add_histogram_data_to_chart(
    chart: Chart,
    histogram: &HistogramData,
    name: &str,
    index: i32,
    max_y: i32,
) -> Chart {
    let x_axis_name = if let Some(descriptive_stats) = &histogram.descriptive_stats {
        format!(
            "min: {}, mean: {}, max: {}",
            descriptive_stats.min_value, descriptive_stats.mean, descriptive_stats.max_value
        )
    } else {
        String::new()
    };

    chart
        .x_axis(
            Axis::new()
                .data(histogram.axis_names())
                .name(x_axis_name)
                .name_text_style(TextStyle::new().font_size(LABEL_FONT_SIZE))
                .name_location(NameLocation::Middle)
                .name_gap(25)
                .grid_index(index),
        )
        .y_axis(
            Axis::new()
                .name(format!("Occurrences (total: {})", histogram.num_values))
                .name_text_style(TextStyle::new().font_size(LABEL_FONT_SIZE))
                .grid_index(index)
                .max(max_y),
        )
        .series(
            Bar::new()
                .data(histogram.occurrences_as_i32())
                .name(name)
                .x_axis_index(index)
                .y_axis_index(index),
        )
}

pub fn create_floxer_vs_minimap_plots(
    data: &DetailedMappedReadsComparison,
    benchmark_folder: &BenchmarkFolder,
    suite_config: &BenchmarkSuiteConfig,
) {
    let num_total_queries = data.general_stats.number_of_queries;
    let minimap_data = &data.minimap_stats_if_minimap_mapped;
    let only_minimap_data = &data.minimap_stats_if_only_minimap_mapped;
    let both_data = &data.minimap_stats_if_both_mapped;
    let floxer_data = &data.floxer_stats_if_floxer_mapped;

    let mapping_status_chart = Chart::new()
        .legend(
            Legend::new()
                .right("10%")
                .left("20%")
                .text_style(TextStyle::new().font_size(20).color("black")),
        )
        .background_color("white")
        .x_axis(
            Axis::new()
                .data(vec![
                    "Minimap",
                    "Floxer",
                    "Both mapped",
                    "Only Minimap",
                    "Only Floxer",
                ])
                .axis_label(AxisLabel::new().font_size(14).color("black")),
        )
        .y_axis(
            Axis::new()
                .name_text_style(TextStyle::new().font_size(25).color("black"))
                .name("#Reads")
                .axis_label(AxisLabel::new().font_size(18).color("black")),
        )
        .series(
            Bar::new()
                .stack("all")
                .name("linear simple mapped")
                .data(vec![
                    minimap_data.num_basic,
                    floxer_data.num_basic,
                    both_data.num_basic,
                    only_minimap_data.num_basic,
                    data.general_stats.minimap_unmapped_and_floxer_mapped,
                ])
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Inside)
                        .font_size(18),
                ),
        )
        .series(
            Bar::new()
                .stack("all")
                .name("linear large clipping mapped")
                .data(vec![
                    minimap_data.num_best_significantly_clipped,
                    floxer_data.num_best_significantly_clipped,
                    both_data.num_best_significantly_clipped,
                    only_minimap_data.num_best_significantly_clipped,
                ])
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Inside)
                        .font_size(18),
                ),
        )
        .series(
            Bar::new()
                .stack("all")
                .name("linear large error rate mapped")
                .data(vec![
                    minimap_data.num_best_high_edit_distance,
                    floxer_data.num_best_high_edit_distance,
                    both_data.num_best_high_edit_distance,
                    only_minimap_data.num_best_high_edit_distance,
                ])
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Inside)
                        .font_size(18),
                ),
        )
        .series(
            Bar::new()
                .stack("all")
                .name("chimeric or inversion mapped")
                .data(vec![
                    minimap_data.num_best_chimeric_or_inversion,
                    floxer_data.num_best_chimeric_or_inversion,
                    both_data.num_best_chimeric_or_inversion,
                    only_minimap_data.num_best_chimeric_or_inversion,
                ])
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Inside)
                        .font_size(18),
                ),
        )
        .series(
            Bar::new()
                .stack("all")
                .name("unmapped")
                .data(vec![
                    num_total_queries - minimap_data.num_queries,
                    num_total_queries - floxer_data.num_queries,
                ])
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Inside)
                        .font_size(18),
                ),
        );

    // 15478 minimap whre did I lose?
    // 15513 floxer
    // TODO what went wrong?

    save_chart(
        mapping_status_chart,
        "mapping_status_comparison".into(),
        800,
        800,
        benchmark_folder,
        suite_config,
    );

    let multiple_mapping_data = vec![
        minimap_data.multiple_mapping,
        floxer_data.multiple_mapping,
        both_data.multiple_mapping,
        only_minimap_data.multiple_mapping,
    ];

    let single_mapping_data = vec![
        data.general_stats.minimap_mapped - minimap_data.multiple_mapping,
        data.general_stats.floxer_mapped - floxer_data.multiple_mapping,
        data.general_stats.both_mapped - both_data.multiple_mapping,
        data.general_stats.floxer_unmapped_and_minimap_mapped - only_minimap_data.multiple_mapping,
    ];

    // TODO change to fing the error rate + largest indel if there is a basic alignment
    let avg_error_rate_data = vec![
        minimap_data.basic_alignments_average_error_rate,
        floxer_data.basic_alignments_average_error_rate,
    ];
    let largest_indel_data = vec![
        minimap_data.basic_average_longest_indel,
        floxer_data.basic_average_longest_indel,
    ];

    let default_offset_str = format!("{GRID_OUTERMOST_OFFSET}%");
    let alignment_characteristics_chart = Chart::new()
        .legend(Legend::new().right("10%"))
        .background_color("white")
        .grid(Grid::new().left(default_offset_str.as_str()).right("70%"))
        .x_axis(
            Axis::new()
                .grid_index(0)
                .data(vec!["Minimap", "Floxer", "Both", "Only Minimap"]),
        )
        .y_axis(Axis::new().grid_index(0).name("number of alignments"))
        .grid(Grid::new().left("35%").right("35%"))
        .x_axis(Axis::new().grid_index(1).data(vec!["Minimap", "Floxer"]))
        .y_axis(
            Axis::new()
                .grid_index(1)
                .name("avg. alignment error rate (only simple)"),
        )
        .grid(Grid::new().left("70%").right(default_offset_str.as_str()))
        .x_axis(Axis::new().grid_index(2).data(vec!["Minimap", "Floxer"]))
        .y_axis(
            Axis::new()
                .grid_index(2)
                .name("avg. largest indel in alignment (only simple)"),
        )
        .series(
            Bar::new()
                .stack("all")
                .x_axis_index(0)
                .y_axis_index(0)
                .name("multiple mapping")
                .data(multiple_mapping_data)
                .label(Label::new().show(true).position(LabelPosition::Inside)),
        )
        .series(
            Bar::new()
                .stack("all")
                .x_axis_index(0)
                .y_axis_index(0)
                .name("single mapping")
                .data(single_mapping_data)
                .label(Label::new().show(true).position(LabelPosition::Inside)),
        )
        .series(
            Bar::new()
                .x_axis_index(1)
                .y_axis_index(1)
                .data(avg_error_rate_data)
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Top)
                        .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                ),
        )
        .series(
            Bar::new()
                .x_axis_index(2)
                .y_axis_index(2)
                .data(largest_indel_data)
                .label(
                    Label::new()
                        .show(true)
                        .position(LabelPosition::Top)
                        .formatter(Formatter::Function(JS_FLOAT_FORMATTER.into())),
                ),
        );

    save_chart(
        alignment_characteristics_chart,
        "alignment_characteristics_comparison".into(),
        1600,
        800,
        benchmark_folder,
        suite_config,
    );
}

fn save_chart(
    chart: Chart,
    plot_name: String,
    width: u32,
    height: u32,
    benchmark_folder: &BenchmarkFolder,
    suite_config: &BenchmarkSuiteConfig,
) {
    let mut renderer = ImageRenderer::new(width, height);

    let mut in_benchmark_folder = benchmark_folder.plot_folder().clone();
    if !in_benchmark_folder.exists() {
        fs::create_dir_all(&in_benchmark_folder).expect("create plot folder in benchmark folder");
    }

    in_benchmark_folder.push(&plot_name);
    in_benchmark_folder.set_extension("svg");
    renderer
        .save(&chart, in_benchmark_folder)
        .expect("failed to save a plot to benchmark folder");

    let mut in_all_plots_folder = suite_config.all_plots_folder();
    in_all_plots_folder.push(plot_name);
    in_all_plots_folder.set_extension("svg");
    renderer
        .save(&chart, in_all_plots_folder)
        .expect("failed to save a plot to all plots folder");
}

fn nice_upper_bound(value: usize) -> usize {
    let max_upper_bound = (value as f64 * 1.2) as usize;

    let exp = ((value as f64).log10() - 1.0)
        .clamp(0.0, f64::INFINITY)
        .round();

    let inc = 10.0f64.powf(exp) as usize;
    let factor = max_upper_bound / inc;

    let upper_bound = inc * factor;

    if upper_bound < value {
        upper_bound + inc
    } else {
        upper_bound
    }
}
