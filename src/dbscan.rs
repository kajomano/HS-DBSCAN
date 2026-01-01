use crate::{proximity::Proximity, types::IndexType};

// TODO: docstring
pub fn dbscan<P: Proximity>(prox: &P, min_pts: IndexType) -> Vec<IndexType> {
    let mut dbscan = Dbscan::new(prox, min_pts);
    dbscan.cluster(prox);
    dbscan.clusters()
}

struct Dbscan {
    min_pts: IndexType,
    states: Vec<PointState>,
}

struct PointState {
    idx: IndexType,
    assigned: bool,
    cluster: IndexType,
    seed: bool,
}

impl Dbscan {
    fn new<P: Proximity>(prox: &P, min_pts: IndexType) -> Self {
        Self {
            min_pts,
            states: (0..prox.len())
                .map(|idx| PointState {
                    idx: idx as IndexType,
                    assigned: false,
                    cluster: 0,
                    seed: false,
                })
                .collect(),
        }
    }

    // TODO: docstring
    fn cluster<P: Proximity>(&mut self, prox: &P) {
        let mut cluster_id: IndexType = 1;

        for idx in 0..self.states.len() {
            unsafe {
                if !self.states.get_unchecked(idx).assigned {
                    if self.expand_cluster(prox, idx, cluster_id) {
                        cluster_id += 1;
                    }
                }
            }
        }
    }

    // TODO: docstring
    unsafe fn expand_cluster<P: Proximity>(
        &mut self,
        prox: &P,
        idx: usize,
        cluster_id: IndexType,
    ) -> bool {
        let prox_query = prox.query(idx);
        for (state, &weight) in self.states.iter_mut().zip(prox_query.iter()) {
            state.seed = weight > 0;
        }

        // Not a core point
        if prox_query.sum() < self.min_pts {
            self.set_cluster_ids_on_seeds(0);
            return false;
        }

        // Core point: all seeds are density reachable
        self.set_cluster_ids_on_seeds(cluster_id);
        unsafe {
            self.states.get_unchecked_mut(idx).seed = false;
        }

        while let Some(idx) = self
            .states
            .iter()
            .filter_map(|state| {
                if state.seed {
                    Some(state.idx as usize)
                } else {
                    None
                }
            })
            .next()
        {
            let prox_query = prox.query(idx);
            if prox_query.sum() >= self.min_pts {
                for idx in prox_query
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &weight)| if weight > 0 { Some(idx) } else { None })
                {
                    unsafe {
                        let state = self.states.get_unchecked_mut(idx);

                        if !state.assigned {
                            state.assigned = true;
                            state.cluster = cluster_id;
                            state.seed = true;
                        } else if state.cluster == 0 {
                            state.cluster = cluster_id;
                        }
                    }
                }
            }

            unsafe {
                self.states.get_unchecked_mut(idx).seed = false;
            }
        }

        true
    }

    // TODO: docstring
    fn set_cluster_ids_on_seeds(&mut self, cluster_id: IndexType) {
        for state in self.states.iter_mut() {
            if state.seed {
                state.cluster = cluster_id;
                state.assigned = true;
            }
        }
    }

    fn clusters(self) -> Vec<IndexType> {
        self.states.into_iter().map(|state| state.cluster).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{dbscan::Dbscan, proximity::Proximity, types::IndexType};
    use nalgebra::{Dyn, OVector, VectorView};
    use rstest::rstest;

    struct TestProximity {
        n_1: usize,
        n_2: usize,
        query_1: OVector<IndexType, Dyn>,
        query_2: OVector<IndexType, Dyn>,
    }

    impl TestProximity {
        pub fn new(n_1: usize, n_2: usize, in_proximity: bool) -> Self {
            Self {
                n_1,
                n_2,
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

        fn len(&self) -> usize {
            self.n_1 + self.n_2
        }
    }

    #[rstest]
    #[case(1, 1, false, 0, 0)]
    #[case(1, 1, true, 0, 0)]
    #[case(5, 1, false, 1, 0)]
    #[case(1, 5, false, 0, 1)]
    #[case(5, 1, true, 1, 1)]
    #[case(1, 5, true, 1, 1)]
    #[case(5, 5, false, 1, 2)]
    fn test_dbscan(
        #[case] n_1: usize,
        #[case] n_2: usize,
        #[case] in_proximity: bool,
        #[case] expected_label_1: IndexType,
        #[case] expected_label_2: IndexType,
    ) {
        let prox = TestProximity::new(n_1, n_2, in_proximity);
        let mut dbscan = Dbscan::new(&prox, 3);

        dbscan.cluster(&prox);

        let mut expected: Vec<IndexType> = vec![expected_label_1; n_1];
        expected.append(&mut vec![expected_label_2; n_2]);

        let assigned: Vec<bool> = dbscan.states.iter().map(|state| state.assigned).collect();
        let clusters: Vec<IndexType> = dbscan.clusters();

        assert_eq!(assigned, vec![true; n_1 + n_2]);
        assert_eq!(clusters, expected);
    }
}
