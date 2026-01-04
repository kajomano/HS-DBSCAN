//! To run the application:
//! ```
//! cargo run --release
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
    generate::{Generate, UniformBox},
    hs_dbscan,
};
use std::{fs::File, path::Path};

fn main() -> Result<()> {
    let config: HsDbscanConfig = serde_json::from_reader(File::open(Path::new("./config.json"))?)?;
    let input = UniformBox::<2>::test_default().generate(1000)?;
    let cluster_ids = hs_dbscan(&input, &config)?;

    // Plotting
    // https://github.com/dataviz-rs/dataviz-examples/blob/main/pixelscattergraphdisplay/src/main.rs
    let mut datasets: Vec<ScatterGraphDataset> = (0..(cluster_ids.max() + 1))
        .map(|cluster_id| {
            if cluster_id == 0 {
                return ScatterGraphDataset::new(
                    [128, 128, 128],
                    "Noise",
                    ScatterDotType::Circle(2),
                );
            }

            // TODO: colors
            ScatterGraphDataset::new(
                [220, 0, 0],
                &format!("Cluster {cluster_id}"),
                ScatterDotType::Circle(2),
            )
        })
        .collect();

    for (point, id) in input.row_iter().zip(cluster_ids.iter()) {
        datasets[*id as usize].add_point((point[(0, 0)], point[(0, 1)]));
    }

    let figure_config = FigureConfig {
        font_label: Some("arial.ttf".to_string()),
        font_title: Some("arial.ttf".to_string()),
        color_grid: [255, 255, 255],
        ..Default::default()
    };

    let mut canvas = PixelCanvas::new(800, 600, [255, 255, 255], 80);
    let mut scatter_graph = ScatterGraph::new("TODO", "", "", figure_config);

    for dataset in datasets.into_iter() {
        scatter_graph.add_dataset(dataset);
    }

    scatter_graph.draw(&mut canvas);
    canvas.save_as_image("plots/scatter_graph.png");

    Ok(())
}
