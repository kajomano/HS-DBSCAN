//! To run the application:
//! ```
//! cargo run --release --features visualize --bin plotter
//! ```

use dataviz::figure::{
    canvas::pixelcanvas::PixelCanvas,
    configuration::figureconfig::FigureConfig,
    datasets::{dataset::Dataset, scattergraphdataset::ScatterGraphDataset},
    drawers::drawer::Drawer,
    figuretypes::scattergraph::ScatterGraph,
    utilities::scatterdottype::ScatterDotType,
};
use eyre::Result;
use hs_dbscan::{
    config::{HsDbscanConfig, test::TestDefault},
    generate::{Generate, TwoUniformSpheres, UniformBox, UniformSphere},
    hs_dbscan,
    types::{FloatMatrixType, IndexType, IndexVectorType},
};
use std::{
    fs::{File, create_dir_all},
    path::Path,
};

const COLORS: [[u8; 3]; 5] = [
    [31, 119, 180],
    [255, 127, 14],
    [44, 160, 44],
    [214, 39, 40],
    [148, 103, 189],
];

/// Rotates through the colors.
fn select_color(cluster_id: IndexType) -> [u8; 3] {
    COLORS[((cluster_id) as usize) % COLORS.len()]
}

/// Creates a scatterplot datasets for each cluster.
fn create_datasets<const NDIMS: usize>(
    input: &FloatMatrixType<NDIMS>,
    cluster_ids: &IndexVectorType,
) -> Vec<ScatterGraphDataset> {
    // Create datasets
    let mut datasets: Vec<ScatterGraphDataset> = (0..(cluster_ids.max() + 1))
        .map(|cluster_id| {
            if cluster_id == 0 {
                return ScatterGraphDataset::new(
                    [128, 128, 128],
                    "Noise",
                    ScatterDotType::Circle(6),
                );
            }

            ScatterGraphDataset::new(
                select_color(cluster_id - 1),
                &format!("Cluster {cluster_id}"),
                ScatterDotType::Circle(6),
            )
        })
        .collect();

    // Assign points to datasets
    for (point, id) in input.column_iter().zip(cluster_ids.iter()) {
        datasets[*id as usize].add_point((point[(0, 0)] as f64, point[(1, 0)] as f64));
    }

    datasets
}

/// Plot a scatter plot (taken from [example](https://github.com/dataviz-rs/dataviz-examples/blob/main/pixelscattergraphdisplay/src/main.rs)).
fn plot_scatter(datasets: Vec<ScatterGraphDataset>, file_name: &str, plot_title: &str) {
    let figure_config = FigureConfig {
        font_label: Some("resources/arial.ttf".to_string()),
        font_title: Some("resources/arial.ttf".to_string()),
        color_grid: [255, 255, 255],
        ..Default::default()
    };

    let mut canvas = PixelCanvas::new(1920, 1920, [255, 255, 255], 80);
    let mut scatter_graph = ScatterGraph::new(plot_title, "", "", figure_config);

    for dataset in datasets.into_iter() {
        scatter_graph.add_dataset(dataset);
    }

    scatter_graph.draw(&mut canvas);
    canvas.save_as_image(&format!("plots/{}.png", file_name));
}

fn generate_plot<G: Generate<2>>(generator: G, n_pts: usize) -> Result<()> {
    let config: HsDbscanConfig = serde_json::from_reader(File::open(Path::new("./config.json"))?)?;

    let input = generator.generate(n_pts)?;
    let cluster_ids = hs_dbscan(&input, &config)?;

    let file_name = format!("{}_{}", generator, n_pts);
    let plot_title = format!(
        "{}, n_pts: {}, config: {}",
        generator,
        n_pts,
        &serde_json::to_string(&config)?
    );

    plot_scatter(
        create_datasets(&input, &cluster_ids),
        &file_name,
        &plot_title,
    );

    Ok(())
}

fn main() -> Result<()> {
    // Recommended:
    // n_pts: 100   - min_pts: 10
    // n_pts: 1000  - min_pts: 70
    // n_pts: 10000 - min_pts: 500

    create_dir_all(Path::new("./plots"))?;

    generate_plot(UniformBox::test_default(), 10000)?;
    generate_plot(UniformSphere::test_default(), 10000)?;
    generate_plot(TwoUniformSpheres::test_default(), 10000)
}
