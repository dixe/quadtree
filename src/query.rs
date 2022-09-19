use crate::data::*;
use crate::*;

pub enum Query {
    Point(QuadPoint),
    Rect(Rect)
}

impl Query {
    pub fn point(x: i32, y: i32) -> Self {
        Query::Point(QuadPoint { x, y })
    }

    pub fn rect(rect: Rect) -> Self {
        Query::Rect(rect)
    }
}



// Public interface for query
impl<'a, T: std::fmt::Debug> QuadTree<T> {

    pub fn query(&self, query: &Query) -> Vec::<&T> {

        let root_rect = self.root_rect.clone();

        let mut element_ids = std::collections::HashSet::new();
        self.query_node_box(0, &root_rect, query, &mut element_ids);

        let mut res = Vec::new();

        for index in element_ids.into_iter() {
            res.push(&self.data[index as usize]);
        }

        res
    }
}
