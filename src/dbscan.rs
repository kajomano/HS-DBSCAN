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
        idx: usize,
        cluster_id: IndexType,
    ) -> bool {
        todo!()
    }
}
