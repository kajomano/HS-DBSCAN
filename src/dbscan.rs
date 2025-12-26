use crate::{
    proximity::Proximity,
    types::{IndexType, MatrixType},
};
use std::collections::HashSet;

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
    use crate::{
        dbscan::Dbscan,
        proximity::Proximity,
        types::{IndexType, MatrixType},
    };
    use nalgebra::{Dyn, OVector, VectorView};
    use rstest::rstest;

    struct TestProximity {
        n_1: IndexType,
        query_1: OVector<IndexType, Dyn>,
        query_2: OVector<IndexType, Dyn>,
    }

    impl TestProximity {
        pub fn new(n_1: usize, n_2: usize, in_proximity: bool) -> Self {
            Self {
                n_1: n_1 as IndexType,
                query_1: OVector::<IndexType, Dyn>::from_fn(n_1 + n_2, |r, _| {
                    if r < n_1 || in_proximity { 1 } else { 0 }
                }),
                query_2: OVector::<IndexType, Dyn>::from_fn(n_1 + n_2, |r, _| {
                    if r >= n_1 || in_proximity { 1 } else { 0 }
                }),
            }
        }
    }

    impl Proximity for TestProximity {
        fn query<'a>(&'a self, query_idx: IndexType) -> VectorView<'a, IndexType, Dyn> {
            if query_idx < self.n_1 {
                self.query_1.column(0)
            } else {
                self.query_2.column(0)
            }
        }
    }

    #[rstest]
    #[case(1, 1, false, 3, 0, 0)]
    #[case(1, 1, true, 3, 0, 0)]
    // #[case(5, 1, false, 3, 10, 0)]
    // #[case(1, 5, false, 3, 0, 20)]
    // #[case(5, 1, true, 3, 10, 10)]
    // #[case(1, 5, true, 3, 10, 10)]
    fn test_dbscan(
        #[case] n_1: usize,
        #[case] n_2: usize,
        #[case] in_proximity: bool,
        #[case] min_pts: IndexType,
        #[case] expected_label_1: IndexType,
        #[case] expected_label_2: IndexType,
    ) {
        let prox = TestProximity::new(n_1, n_2, in_proximity);
        let input = MatrixType::<2>::zeros(n_1 + n_2);
        let mut dbscan = Dbscan::new(&input, min_pts);

        dbscan.expand_cluster(&prox, 0, 10);
        dbscan.expand_cluster(&prox, n_1 as IndexType, 20);

        let mut expected: Vec<IndexType> = vec![expected_label_1; n_1];
        expected.append(&mut vec![expected_label_2; n_2 as usize]);

        assert_eq!(dbscan.clustered, vec![true; (n_1 + n_2) as usize]);
        assert_eq!(dbscan.clusters, expected);
    }
}
