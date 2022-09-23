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
impl<'a, T: std::fmt::Debug + Copy> QuadTree<T> {

    pub fn query_p(&mut self, query_p: Point, omit_elm: i32, output: &mut Vec<T>){

        self.query(Point::to_rect(query_p), omit_elm, output);

    }

    pub fn query(&mut self, query_r: Rect, omit_elm: i32, output: &mut Vec<T>){

         if self.query_tmp_buffer.len() < self.elm_rects.elements_count() as usize {
             for _ in 0..(self.elm_rects.elements_count() as usize - self.query_tmp_buffer.len() as usize) {
                 self.query_tmp_buffer.push(false);
             }
         }

         let root_rect = self.root_rect;
         self.query_node_box_rect(root_rect, query_r, omit_elm, output);

         // clear tmp buffer

         for i in 0..self.query_tmp_buffer.len() {
             self.query_tmp_buffer[i] = false;
         }

    }

    fn query_node_box_rect(&mut self, node_rect: Rect, query_r: Rect, omit_elm: i32, data_vec: &mut  Vec::<T>) {

        let leaves = self.find_leaves(0, self.root_rect, query_r, 0);

        for &leaf in &leaves {
            self.find_element(leaf.node_index, node_rect, query_r, omit_elm, data_vec);
        }
    }


    fn find_element(&mut self, node_index: i32, node_rect: Rect, query_r: Rect, omit_elm: i32, data_vec: &mut  Vec::<T>) {

        let leaf_node = &self.nodes[node_index];

            let mut elm_node_index = leaf_node.first_child;

            while elm_node_index != -1 {
                let elm_node = &self.element_nodes[elm_node_index];
                let element_id = elm_node.elm_id;
                let elm_rect = &self.elm_rects[element_id];
                // Not omit and not already added to output and intersect query
                if omit_elm != element_id && !self.query_tmp_buffer[element_id as usize] && query_r.intersect(elm_rect.rect) {
                    //println!("{:?}", (query_r, elm_rect.rect));
                    // add to found element for this query
                    self.query_tmp_buffer[element_id as usize] = true;

                    // TODO: validate that output is also what we expect, should it be elment_id or data_id??
                    // Should be element_id or data[data_id], since we return eelment_id to user of tree.
                    // Or it should be the actual data, maybe as ref??
                    // add to output
                    data_vec.push(self.data[self.elm_rects[element_id].data_id]);
                    return;
                }
                elm_node_index = elm_node.next;
            }
    }
}
