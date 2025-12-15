use ndarray::{Array2};

type Float = f64;

enum Distance {
    L2(Float),
}

struct RfDbscan {
    raster_res: Float,
    eps: Distance,
    min_pts: usize,
}

struct ClusteringResult {
    clusters: Vec<Array2<Float>>,
    remaining: Array2<Float>,
}

impl RfDbscan {
    fn cluster(&self, input: Array2<Float>) -> ClusteringResult {
        ClusteringResult {
            clusters: vec![],
            remaining: Array2::zeros((0, 2)),
        }
    }
}

fn main() {
    let rf_dbscan = RfDbscan {
        raster_res: 1.0,
        eps: Distance::L2(1.0),
        min_pts: 10,
    };

    let input = Array2::<Float>::zeros((3, 2));

    rf_dbscan.cluster(input);
}
