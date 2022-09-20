use crate::data::*;
use crate::*;

pub enum Query {
    Point(Point),
    Rect(Rect)
}

impl Query {
    pub fn point(x: i32, y: i32) -> Self {
        Query::Point(Point { x, y })
    }

    pub fn rect(rect: Rect) -> Self {
        Query::Rect(rect)
    }
}



// Public interface for query
impl<'a, T: std::fmt::Debug + std::cmp::Ord + Copy> QuadTree<T> {

    pub fn query(&self, query: &Query) -> Vec::<&T> {

        let root_rect = self.root_rect.clone();

        let mut element_ids : Vec::<i32> = Vec::with_capacity(self.data.active() as usize);
        self.query_node_box(0, &root_rect, query, &mut element_ids);

        element_ids.sort();
        element_ids.dedup();
        let mut res = Vec::with_capacity(element_ids.len());

        for index in element_ids.into_iter() {
            res.push(&self.data[index]);
        }

        res


    }
}
