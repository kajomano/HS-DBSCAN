use std::collections::HashSet;

use crate::{
    proximity::Proximity,
    types::{IndexType, MatrixType},
};

pub struct Dbscan {
    min_pts: IndexType,
    clustered: Vec<bool>,
    clusters: Vec<IndexType>,
}

impl Dbscan {
    pub fn new<const NCOLS: usize>(input: &MatrixType<NCOLS>, min_pts: IndexType) -> Self {
        Self {
            min_pts,
            clustered: vec![false; input.nrows()],
            clusters: vec![0; input.nrows()],
        }
    }

    // TODO: docstring
    pub fn expand_cluster<P: Proximity>(
        &mut self,
        prox: &P,
        idx: IndexType,
        cluster_id: IndexType,
    ) -> bool {
        let prox_query = prox.query(idx);
        let mut seeds: HashSet<IndexType> = prox_query
            .iter()
            .enumerate()
            .filter_map(|(weight, i)| if weight > 0 { Some(*i) } else { None })
            .collect();

        // Not a core point
        if prox_query.sum() < self.min_pts {
            self.set_cluster_ids(&seeds, 0);
            return false;
        }

        // Core point: all seeds are density reachable
        self.set_cluster_ids(&seeds, cluster_id);
        seeds.remove(&idx);

        while let Some(idx) = seeds.iter().next().cloned() {
            seeds.remove(&idx);

            let prox_query = prox.query(idx);
            if prox_query.sum() >= self.min_pts {
                for idx in prox_query.iter() {
                    unsafe {
                        let clustered = self.clustered.get_unchecked_mut(*idx as usize);
                        let cluster = self.clusters.get_unchecked_mut(*idx as usize);

                        if *clustered == false || *cluster == 0 {
                            if *clustered == false {
                                seeds.insert(*idx);
                            }

                            *clustered = true;
                            *cluster = cluster_id;
                        }
                    }
                }
            }
        }

        true
    }

    fn set_cluster_ids(&mut self, idxs: &HashSet<IndexType>, cluster_id: IndexType) {
        unsafe {
            for idx in idxs {
                *self.clustered.get_unchecked_mut(*idx as usize) = true;
                *self.clusters.get_unchecked_mut(*idx as usize) = cluster_id;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::{OMatrix, RowVector2};
    use rstest::rstest;
    use crate::types::{IndexType, FloatType, MatrixType};

    #[rstest]
    #[case([0.0, 0.0], 1, [0.0, 0.0], 1, 1, [0, 0])]
    // #[case(L1Norm, [1.0, -2.0], 3.0)]
    // #[case(L2Norm, [1.0, 2.0], f(5.0).sqrt())]
    // #[case(L2SquaredNorm, [1.0, 2.0], 5.0)]
    // #[case(LinfNorm, [1.0, 2.0], 2.0)]
    // #[case(LinfNorm, [1.0, -2.0], 2.0)]
    fn test_dbscan(
        #[case] points_1: [FloatType; 2],
        #[case] n_1: usize,
        #[case] points_2: [FloatType; 2],
        #[case] n_2: usize,
        #[case] min_pts: IndexType,
        #[case] expected_labels: [IndexType; 2]
    ) {
        let mut rows = vec![RowVector2::from_row_slice(&points_1); n_1];
        rows.append(&mut vec![RowVector2::from_row_slice(&points_1); n_2]);

        let input = MatrixType::from_rows(&rows);

        // TODO: ITT
        todo!()
    }
}