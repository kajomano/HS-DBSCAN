use crate::{
    proximity::Proximity,
    types::{IndexType, MatrixType},
};
use std::collections::HashSet;

pub struct Dbscan {
    min_pts: IndexType,
    assigned: Vec<bool>,
    clusters: Vec<IndexType>,
}

impl Dbscan {
    pub fn new<const NCOLS: usize>(input: &MatrixType<NCOLS>, min_pts: IndexType) -> Self {
        Self {
            min_pts,
            assigned: vec![false; input.nrows()],
            clusters: vec![0; input.nrows()],
        }
    }

    // TODO: docstring
    // TODO: return clustering
    pub fn cluster<P: Proximity>(&mut self, prox: &P) {
        let mut cluster_id: IndexType = 1;

        for idx in 0..self.assigned.len() {
            if !self.assigned[idx] {
                if self.expand_cluster(prox, idx, cluster_id) {
                    cluster_id += 1;
                }
            }
        }
    }

    // TODO: docstring
    fn expand_cluster<P: Proximity>(
        &mut self,
        prox: &P,
        idx: usize,
        cluster_id: IndexType,
    ) -> bool {
        assert!((idx as usize) < self.clusters.len());

        let prox_query = prox.query(idx);
        let mut seeds: HashSet<usize> = prox_query
            .iter()
            .enumerate()
            .filter_map(|(idx, weight)| if *weight > 0 { Some(idx) } else { None })
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
                for (idx, weight) in prox_query.iter().enumerate() {
                    if *weight > 0 {
                        unsafe {
                            let assigned = self.assigned.get_unchecked_mut(idx);
                            let cluster = self.clusters.get_unchecked_mut(idx);

                            if *assigned == false || *cluster == 0 {
                                if *assigned == false {
                                    seeds.insert(idx);
                                }

                                *assigned = true;
                                *cluster = cluster_id;
                            }
                        }
                    }
                }
            }
        }

        true
    }

    fn set_cluster_ids(&mut self, idxs: &HashSet<usize>, cluster_id: IndexType) {
        unsafe {
            for idx in idxs {
                *self.assigned.get_unchecked_mut(*idx) = true;
                *self.clusters.get_unchecked_mut(*idx) = cluster_id;
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
        n_1: usize,
        query_1: OVector<IndexType, Dyn>,
        query_2: OVector<IndexType, Dyn>,
    }

    impl TestProximity {
        pub fn new(n_1: usize, n_2: usize, in_proximity: bool) -> Self {
            Self {
                n_1: n_1,
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
        fn query<'a>(&'a self, query_idx: usize) -> VectorView<'a, IndexType, Dyn> {
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
    #[case(5, 1, false, 3, 1, 0)]
    #[case(1, 5, false, 3, 0, 1)]
    #[case(5, 1, true, 3, 1, 1)]
    #[case(1, 5, true, 3, 1, 1)]
    #[case(5, 5, false, 3, 1, 2)]
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

        dbscan.cluster(&prox);

        let mut expected: Vec<IndexType> = vec![expected_label_1; n_1];
        expected.append(&mut vec![expected_label_2; n_2 as usize]);

        assert_eq!(dbscan.assigned, vec![true; (n_1 + n_2) as usize]);
        assert_eq!(dbscan.clusters, expected);
    }
}
