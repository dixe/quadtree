use crate::data::*;
use crate::*;

#[derive(Debug,Clone,Copy)]
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

     pub fn query(&mut self, query_r: Rect, omit_elm: i32, output: &mut Vec<i32>){

         if self.query_tmp_buffer.len() < self.elm_rects.len() as usize {
             for _ in 0..(self.elm_rects.len() as usize - self.query_tmp_buffer.len() as usize) {
                 self.query_tmp_buffer.push(false);
             }
         }

         let root_rect = self.root_rect;
         self.query_node_box_rect(0, root_rect, query_r, omit_elm, output);

         // clear tmp buffer
         for &mut element_index in output {
             self.query_tmp_buffer[element_index as usize] = false;
         }

    }
}
