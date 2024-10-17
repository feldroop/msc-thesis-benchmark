use std::fs;

use crate::{
    config::BenchmarkSuiteConfig,
    floxer::{HistogramData, ResourceMetrics},
    folder_structure::BenchmarkFolder,
};

use charming::{
    component::{Axis, Grid, Legend, Title},
    element::{NameLocation, TextStyle},
    series::Bar,
    Chart, ImageRenderer,
};

static TITLE_FONT_SIZE: u8 = 25;
static SUBTITLE_FONT_SIZE: u8 = 20;
static LABEL_FONT_SIZE: u8 = 15;
static GRID_OUTERMOST_OFFSET: usize = 6;

pub fn plot_resource_metrics<'a>(
    benchmark_name: &str,
    metrics_and_names_of_runs: impl Iterator<Item = (&'a ResourceMetrics, &'a str)>,
    benchmark_folder: &BenchmarkFolder,
    suite_config: &BenchmarkSuiteConfig,
) {
    let offset_str = "54%";

    let mut chart = Chart::new()
        .title(Title::new().text(format!("Resource Usage For {}", benchmark_name)))
        .legend(Legend::new().right("10%"))
        .grid(Grid::new().right(offset_str))
        .grid(Grid::new().left(offset_str))
        .background_color("white")
        .x_axis(
            Axis::new()
                .data(vec!["Wall Time", "User CPU Time", "System CPU Time"])
                .grid_index(0),
        )
        .y_axis(Axis::new().name("Seconds").grid_index(0))
        .x_axis(
            Axis::new()
                .data(vec!["Peak Memory Usage", "Average Memory Usage"])
                .grid_index(1),
        )
        .y_axis(Axis::new().name("Kilobytes").grid_index(1));

    for (metrics, name) in metrics_and_names_of_runs {
        chart = chart
            .series(
                Bar::new()
                    .data(vec![
                        metrics.wall_clock_seconds,
                        metrics.user_cpu_seconds,
                        metrics.system_cpu_seconds,
                    ])
                    .name(name)
                    .x_axis_index(0)
                    .y_axis_index(0),
            )
            .series(
                Bar::new()
                    .data(vec![
                        metrics.peak_memory_kilobytes as i64,
                        metrics.average_memory_kilobytes as i64,
                    ])
                    .name(name)
                    .x_axis_index(1)
                    .y_axis_index(1),
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
        .title(
            Title::new()
                .text(title)
                .left("center")
                .text_style(TextStyle::new().font_size(TITLE_FONT_SIZE)),
        )
        .legend(
            Legend::new()
                .right("right")
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
                .grid_index(index),
        )
        .series(
            Bar::new()
                .data(histogram.occurrences_as_i32())
                .name(name)
                .x_axis_index(index)
                .y_axis_index(index),
        )
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
