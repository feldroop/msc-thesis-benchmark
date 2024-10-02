use std::{fs, path::Path};

use crate::{
    floxer::{FloxerResult, HistogramData, ResourceMetrics},
    folder_structure::BenchmarkFolder,
};

use charming::{
    component::{Axis, Grid, Legend, Title},
    series::Bar,
    Chart, ImageRenderer,
};

pub fn plot_resource_metrics<'a>(
    benchmark_name: &str,
    metrics_and_names_of_runs: impl Iterator<Item = (&'a ResourceMetrics, &'a str)>,
    benchmark_folder: &BenchmarkFolder,
    root_output_folder: &Path,
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
        root_output_folder,
    );
}

pub fn plot_general_floxer_info(
    benchmark_name: &str,
    floxer_results: &[FloxerResult],
    benchmark_folder: &BenchmarkFolder,
    root_output_folder: &Path,
) {
    let big_offset_str = "70%";
    let small_offset_str = "35%";

    let mut chart = Chart::new()
        .title(Title::new().text(format!("General info For {}", benchmark_name)))
        .legend(Legend::new().right("10%").left("40%"))
        .background_color("white");

    // we calculate with 110%, because the values are offsets and the extra 10% adds a margin
    let column_width = 110f64 / floxer_results.len() as f64;
    for (base_index, floxer_result) in floxer_results.iter().enumerate() {
        let left_offset_str = format!("{}%", (base_index as f64 * column_width) as usize);
        let right_offset_str = format!(
            "{}%",
            ((floxer_results.len() - 1 - base_index) as f64 * column_width) as usize
        );

        chart = chart
            .grid(add_horizontal_position_to_grid(
                Grid::new().bottom(big_offset_str),
                &left_offset_str,
                &right_offset_str,
            ))
            .grid(add_horizontal_position_to_grid(
                Grid::new().bottom(small_offset_str).top(small_offset_str),
                &left_offset_str,
                &right_offset_str,
            ))
            .grid(add_horizontal_position_to_grid(
                Grid::new().top(big_offset_str),
                &left_offset_str,
                &right_offset_str,
            ));

        let base_index = base_index as i32;
        chart = add_histogram_to_chart(
            chart,
            &floxer_result.stats.query_lengths,
            "Query lengths",
            base_index * 3,
        );
        chart = add_histogram_to_chart(
            chart,
            &floxer_result.stats.alignments_per_query,
            "Alignments Per Query",
            base_index * 3 + 1,
        );
        chart = add_histogram_to_chart(
            chart,
            &floxer_result.stats.alignments_edit_distance,
            "Alignments Edit Distance",
            base_index * 3 + 2,
        );
    }

    save_chart(
        chart,
        format!("{benchmark_name}_general_info"),
        600 * floxer_results.len() as u32,
        1200,
        benchmark_folder,
        root_output_folder,
    );
}

fn add_horizontal_position_to_grid(
    mut grid: Grid,
    left_offset_str: &str,
    right_offset_str: &str,
) -> Grid {
    if left_offset_str != "0%" {
        grid = grid.left(left_offset_str);
    }

    if right_offset_str != "0%" {
        grid = grid.right(right_offset_str);
    }

    grid
}

fn add_histogram_to_chart(
    chart: Chart,
    histogram: &HistogramData,
    name: &str,
    index: i32,
) -> Chart {
    chart
        .x_axis(Axis::new().data(histogram.axis_names()).grid_index(index))
        .y_axis(Axis::new().name("Occurrences").grid_index(index))
        .series(
            Bar::new()
                .data(histogram.occurrences_as_i32())
                .name(format!("{name} (total: {})", histogram.num_values))
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
    root_output_folder: &Path,
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

    let mut in_all_plots_folder = crate::folder_structure::all_plots_folder(root_output_folder);
    in_all_plots_folder.push(plot_name);
    in_all_plots_folder.set_extension("svg");
    renderer
        .save(&chart, in_all_plots_folder)
        .expect("failed to save a plot to all plots folder");
}
