use std::fmt;
pub mod free_list;
use free_list::FreeList;
use std::collections::VecDeque;

mod query;
mod data;

pub use self::data::*;
pub use self::query::*;

// From answer here: https://stackoverflow.com/questions/41946007/efficient-and-well-explained-implementation-of-a-quadtree-for-2d-collision-dte

use data::*;
use query::*;

struct FindLeaves {
    node_id: i32,
    rect: Rect,
    depth: i32
}

pub struct QuadTree<T>{

    // All quads of elements in the quadtree
    elm_rects: FreeList<ElmRect>,

    // All elementNodes in quadTree
    // Elements nodes refer to elements
    element_nodes: FreeList<ElmRectNode>,


    // All nodes in quadTree
    // First node is the root
    // leafs are where count > 0 and then first child is index into element_nodes
    // Has to be vec, since otherwise we cannot guarantee that we can get 4 consecutive nodes
    // Or can we??
    nodes: FreeList::<Node>,


    // Actual data inserted into tree
    data: FreeList::<T>,

    // Rect for the root
    // All sub rects are computed on the fly in integers
    root_rect: Rect,

    max_depth: i32,
    elements_per_node: i32,

    // buffer for storing querying, to store elements already found
    query_tmp_buffer: Vec::<bool>,
}



// Public interface
impl<'a, T: std::fmt::Debug> QuadTree<T> {

    pub fn new(rect: Rect) -> Self {

        let mut nodes = FreeList::new();

        nodes.insert(Node {
            first_child: -1,
            count: 0,
        });

        QuadTree {
            elm_rects: FreeList::new(),
            element_nodes: FreeList::new(),
            nodes,
            data: FreeList::new(),
            root_rect: rect,
            max_depth: 10,
            elements_per_node: 300,
            query_tmp_buffer: vec![],
        }
    }

    pub fn insert(&mut self, element: T, element_rect: Rect) ->  i32 {

        //println!("inserting {:?}", element_rect);
        // check if we can insert into root
        let data_id = self.data.insert(element);

        let element_id = self.elm_rects.insert(ElmRect {
            data_id: data_id,
            rect: element_rect.clone()
        });

        //println!("Inserting node for element with id: {:?}", element_id);
        self.node_insert(element_id, 0, self.root_rect, 0);

        element_id
    }

    pub fn set_elements_per_node(&mut self, npc: i32) {
        self.elements_per_node = i32::max(1, npc);
    }


    /// Removes an element from the tree. Does not restructure the tree see ['cleanup()']
    pub fn remove(&mut self, element_id: i32) {
        let elm = &self.elm_rects[element_id];
        let leaves = self.find_leaves(0, self.root_rect, elm.rect, 0);

        for &leaf in &leaves {
            let leaf_index = leaf.node_index;
            let leaf_node = &mut self.nodes[leaf_index];

            let mut element_index = leaf_node.first_child;
            let mut prev_index = -1;

            // only walk until we find element
            while element_index != -1 && self.element_nodes[element_index].elm_id != element_id {
                prev_index = element_index;
                element_index = self.element_nodes[element_index].next;

            }

            if element_index != -1 {  // elment found at element_index
                assert_eq!(self.element_nodes[element_index].elm_id, element_id);


                let next_index = self.element_nodes[element_index].next;

                if prev_index == -1 { // found element is first child, so se node to next
                    self.nodes[leaf_index].first_child = next_index;
                } else {
                    // in middle set
                    self.element_nodes[prev_index].next = next_index;
                }

                //println!("{:?}", (prev_index, element_index, next_index));
                self.element_nodes.erase(element_index);
                self.nodes[leaf_index].count -= 1;
            }
        }

        // also data? but that could be slow???
        self.data.erase(self.elm_rects[element_id].data_id);
        self.elm_rects.erase(element_id);

    }




    /// Clean the tree by making branches with only empty leaf children into leafs
    /// Only does one level per call.
    pub fn cleanup(&mut self) {

        let mut to_process = VecDeque::new();

        if self.nodes[0].is_branch() {
            to_process.push_back(0);
        }

        let mut to_delete = vec![] ;

        while let Some(node_id) = to_process.pop_front() {

            let branch = &self.nodes[node_id];

            let mut empty_children = 0;
            for i in 0..4 {

                let child_index = branch.first_child + i;
                let child = &self.nodes[child_index];

                if child.is_branch() {
                    to_process.push_back(child_index);
                }
                else if child.count == 0 {
                   empty_children += 1;
                }
            }

            if empty_children == 4 {
                to_delete.push(node_id);
            }

        }

        for node_id in to_delete {
            let mut branch = &mut self.nodes[node_id];
            let first_child = branch.first_child;

            // -1 for no children
            // count = 0 for leaf
            branch.first_child = -1;
            branch.count = 0;

            // Delete order is important. Since last to be deleted
            // is also first to be reused in free list. Thus delete in this order
            // ensure that when we split next time, we still get 4 consecutive nodes
            self.nodes.erase(first_child + 3);
            self.nodes.erase(first_child + 2);
            self.nodes.erase(first_child + 1);
            self.nodes.erase(first_child );

        }
    }

    /// Clear all data from the tree. Does not clear the structure. So inserting roughly the same data is
    /// fast since we already have the nodes ready
    pub fn clear(&mut self) {

        self.elm_rects.clear();
        //self.element_nodes.clear();
        self.data.clear();

        self.element_nodes.clear();

        //println!("{:?}", (self.element_node_lists.data_len(), self.element_node_lists.elements_count(),self.nodes.elements_count()));

        // using len is not correct. Since that is active elements
        for i in 0..self.nodes.data_len() {
            if self.nodes[i].is_leaf() {
                self.nodes[i].count = 0;
                self.nodes[i].first_child = -1;
            }
        }
    }

    pub fn max_element_id(&self) -> usize {
        self.elm_rects.data_len() as usize
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Leaf {
    pub node_index: i32,
    pub depth: i32,
    pub rect: Rect,
}

struct InsertProcess {
    element_id: i32,
    node_index: i32,
    node_rect: Rect,
    depth: i32
}
// Private functions
impl<'a, T: std::fmt::Debug> QuadTree<T> {


    fn find_leaves(&self, node_index: i32, node_rect: Rect, search_rect: Rect, depth: i32) -> Vec::<Leaf> {
        let mut res = vec![];


        // start at input node, at branches see which overlaps with rect, process those too
        // return vec of nodes that rect overlaps
        let mut to_process = VecDeque::new();

        //0 is root
        to_process.push_back(FindLeaves{ node_id: node_index, rect: node_rect, depth});


        while let Some(node_data) = to_process.pop_front() {


            let node = &self.nodes[node_data.node_id];

            // if node is a leaf, push to result
            if self.nodes[node_data.node_id].count != -1 {
                assert!(self.nodes[node_data.node_id].count != -1);

                res.push(Leaf{ node_index: node_data.node_id, depth: node_data.depth, rect: node_data.rect });
            }
            else {

                let locations = node_data.rect.location_quads();

                for i in 0..4 {
                    if locations[i].intersect(search_rect) {
                        to_process.push_back(FindLeaves { node_id: node.first_child + i as i32,
                                                          rect: locations[i],
                                                          depth: node_data.depth + 1 });

                    }
                }
            }

        }


        res
    }




    // insert element_id(index into self.elm_rects) into the node with the given index
    fn node_insert(&mut self, element_id: i32, node_index: i32, node_rect: Rect, depth: i32) {


        let mut to_process = VecDeque::new();

        to_process.push_back(InsertProcess { element_id, node_index, node_rect, depth});

        while let Some(node_data) = to_process.pop_front() {

            let leaves = self.find_leaves(node_index, node_rect, self.elm_rects[element_id].rect, depth);

            for leaf in &leaves {

                let current_first_child = self.nodes[leaf.node_index].first_child;

                // insert into leaf, using current leaf first child as this ones next
                // setting leaf first child to this
                let elm_node_index = self.element_nodes.insert(ElmRectNode {
                    next: current_first_child,
                    elm_id: element_id
                });


                // first child is set to a node that was just created
                assert!(current_first_child != elm_node_index);

                self.nodes[leaf.node_index].first_child = elm_node_index;
                self.nodes[leaf.node_index].count += 1;


                // Split node if too big and not too far down
                if self.nodes[leaf.node_index].count >= self.elements_per_node && leaf.depth < self.max_depth  {


                    // transfer node children to tmp list
                    // TODO: Maybe allocate a tmp buffer for this on self??
                    let mut element_list = FreeList::<i32>::new();

                    while self.nodes[leaf.node_index].first_child != -1 {

                        let index = self.nodes[leaf.node_index].first_child;
                        let next_index = self.element_nodes[index].next;
                        let elm_rect_id =  self.element_nodes[index].elm_id;

                        // store element so we can insert into children
                        element_list.insert(elm_rect_id);

                        // iterate
                        // Maybe use local variable??
                        assert!(self.nodes[leaf.node_index].first_child != next_index);
                        self.nodes[leaf.node_index].first_child = next_index;
                        self.element_nodes.erase(index);

                        assert!(self.nodes[leaf.node_index].first_child != index);
                    }

                    assert!(self.nodes[leaf.node_index].first_child == -1);



                    // allocate 4 children
                    let index = self.nodes.insert(Node::leaf());
                    let index2 = self.nodes.insert(Node::leaf());
                    let index3 = self.nodes.insert(Node::leaf());
                    let index4 = self.nodes.insert(Node::leaf());



                    // make leaf into a node
                    self.nodes[leaf.node_index].first_child = index;
                    self.nodes[leaf.node_index].count = -1;


                    // push current children to be processed (inserted into leaves)
                    for i in 0..element_list.data_len() {
                        to_process.push_back(InsertProcess {
                            element_id: element_list[i],
                            node_index,
                            node_rect: leaf.rect,
                            depth: leaf.depth + 1 });

                        //self.node_insert(element_list[i], node_index, leaf.rect, leaf.depth + 1);
                    }
                }
            }
        }

        //println!("Insert exit");
    }


    fn print(&self) -> String {
        self.print_node(0, 0)
    }


    fn print_node(&self, node_index: i32, indent: usize) -> String {

        if self.nodes[node_index].count >= 0 {
            // leaf

            if self.nodes[node_index].count > 0 {
                let mut child_index = self.nodes[node_index].first_child;

                let mut res = "".to_string();
                while child_index != -1 {
                    let elm_node = &self.element_nodes[child_index];
                    res += &format!(" e({}), idx: {:?} | ", elm_node.elm_id, child_index);
                    child_index = elm_node.next;
                }


                return format!("\n{:indent$}-{}", "", res, indent=indent);
            }
            else {
                return format!("\n{:indent$}-Empty", "", indent=indent );
            }
        }
        else {
            // branch
            let first_index = self.nodes[node_index].first_child;

            let mut res = format!("\n{:indent$}","", indent=indent);
            res += &format!("Branch({})", node_index);


            res += &self.print_node(first_index, indent + 4);
            res += &self.print_node(first_index + 1, indent + 4);
            res += &self.print_node(first_index + 2, indent + 4);
            res += &self.print_node(first_index + 3, indent + 4);

            return res;

        }

    }

}


impl<T: std::fmt::Debug> fmt::Display for QuadTree<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print())
    }
}

impl<T: std::fmt::Debug> fmt::Debug for QuadTree<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }

}



#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn node_locations_all() {

        let node_rect = Rect::from_points(Point {x: -128, y: -128}, Point { x: 128, y: 128} );

        let element_rect = Rect::from_points(Point {x: -10, y: -10}, Point { x: 20, y: 20} );

        let locations = Rect::element_quad_locations(node_rect, element_rect);

        //println!("{:?}", locations);

        assert!(locations[0] && locations[1] && locations[2] && locations[3]);

    }



    #[test]
    fn node_locations_tl() {

        let node_rect = Rect::from_points(Point {x: -128, y: -128}, Point { x: 128, y: 128} );

        let element_rect = Rect::from_points(Point {x: -10, y: 10}, Point { x: -20, y: 20} );

        let locations = Rect::element_quad_locations(node_rect, element_rect);

        //println!("{:?}", locations);

        assert!(locations[0] && !locations[1] && !locations[2] && !locations[3]);

    }


    #[test]
    fn node_locations_tr() {

        let node_rect = Rect::from_points(Point {x: -128, y: -128}, Point { x: 128, y: 128} );

        let element_rect = Rect::from_points(Point {x: 10, y: 10}, Point { x: 20, y: 20} );

        let locations = Rect::element_quad_locations(node_rect, element_rect);

        //println!("{:?}", locations);

        assert!(!locations[0] && locations[1] && !locations[2] && !locations[3]);
    }


    #[test]
    fn node_locations_bl() {

        let node_rect = Rect::from_points(Point {x: -128, y: -128}, Point { x: 128, y: 128} );

        let element_rect = Rect::from_points(Point {x: -10, y: -10}, Point { x: -20, y: -20} );

        let locations = Rect::element_quad_locations(node_rect, element_rect);

        //println!("{:?}", locations);

        assert!(!locations[0] && !locations[1] && locations[2] && !locations[3]);
    }



    #[test]
    fn node_locations_br() {

        let node_rect = Rect::from_points(Point {x: -128, y: -128}, Point { x: 128, y: 128} );

        let element_rect = Rect::from_points(Point {x: 10, y: -10}, Point { x: 20, y: -20} );

        let locations = Rect::element_quad_locations(node_rect, element_rect);

        //println!("{:?}", locations);

        assert!(!locations[0] && !locations[1] && !locations[2] && locations[3]);
    }

    #[test]
    fn insert_2_elm() {



        let rect = Rect::from_points(Point {x: -128, y: -128}, Point { x: 128, y: 128} );

        let mut qt = QuadTree::<f32>::new(rect);

        qt.set_elements_per_node(2);

        let elm1_id = 1.0;
        let elm1_rect = Rect::from_points(Point {x: 10, y: 10}, Point { x: 20, y: 20} );
        qt.insert(elm1_id, elm1_rect);

        let elm2_id = 2.0;
        let elm2_rect = Rect::from_points(Point {x: -10, y: -10}, Point { x: -20, y: -20} );
        qt.insert(elm2_id, elm2_rect);



        let elm2_id = 3.0;
        let elm2_rect = Rect::from_points(Point {x: 0, y: 0}, Point { x: 5, y: 5} );
        qt.insert(elm2_id, elm2_rect);


        let mut res = vec![];

        qt.query_p(Point::new(15,15), -1, &mut res);


        assert_eq!(res.len(), 1);
        vec_compare(&res, vec![1.0]);


        res.clear();

        qt.query_p(Point::new(15,15), -1, &mut res);
        assert_eq!(res.len(), 1);
        vec_compare(&res, vec![1.0]);

    }


      #[test]
    fn insert_clear() {

        let rect = Rect::from_points(Point {x: -128, y: -128}, Point { x: 128, y: 128} );

        let mut qt = QuadTree::new(rect);

        qt.set_elements_per_node(2);



        let elm1 = 1;
        let elm1_rect = Rect::from_points(Point {x: 10, y: 10}, Point { x: 20, y: 20} );
        qt.insert(elm1, elm1_rect);

        let elm2 = 2;
        let elm2_rect = Rect::from_points(Point {x: -10, y: -10}, Point { x: -20, y: -20} );
        qt.insert(elm2, elm2_rect);

        let elm3 = 3;
        let elm3_rect = Rect::from_points(Point {x: 0, y: 0}, Point { x: 5, y: 5} );
        qt.insert(elm3, elm3_rect);


        qt.clear();

        let elm1 = 1;
        let elm1_rect = Rect::from_points(Point {x: 10, y: 10}, Point { x: 20, y: 20} );
        qt.insert(elm1, elm1_rect);

        let elm2 = 2;
        let elm2_rect = Rect::from_points(Point {x: 12, y: 12}, Point { x: 20, y: 20} );
        qt.insert(elm2, elm2_rect);

        let elm3 = 3;
        let elm3_rect = Rect::from_points(Point {x: 0, y: 0}, Point { x: 5, y: 5} );
        qt.insert(elm3, elm3_rect);


        assert!(false);

    }



    fn vec_compare<T>(res: &Vec::<T>, expected: Vec::<T>) where T: fmt::Debug + Copy + PartialOrd {
        let mut values = Vec::<T>::new();

        for &p in res {
            values.push(p);
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(expected, values);
    }

/*
    #[test]
    fn insert_remove_1() {

        let rect = Rect::new(-128, 128, 256, 256);

        let mut qt = QuadTree::<f32>::new(rect);

        let elm0_rect = Rect::new(-2, 2, 1, 1);
        let id0 = qt.insert(0.0, elm0_rect);





        let elm1_rect = Rect::new(2, 2, 1, 1);
        let id1 = qt.insert(1.1, elm1_rect);

        let elm2_rect = Rect::new(2, -2, 1, 1);
        let id2 = qt.insert(2.2, elm2_rect);

        let elm00_rect = Rect::new(-2, -2, 1, 1);
        let id00 = qt.insert(0.1, elm00_rect);

        let elm00_rect = Rect::new(-1, 1, 2, 2);
        let id00 = qt.insert(3.3, elm00_rect);

        qt.remove(id00);

    }


    #[test]
    fn cleanup_1() {

        let rect = Rect::new(-128, 128, 256, 256);

        let mut qt = QuadTree::<f32>::new(rect);

        let elm0_rect = Rect::new(5, 5, 1, 1);
        let id0 = qt.insert(5.0, elm0_rect);

        let elm0_rect = Rect::new(-100, -100, 1, 1);
        let id1 = qt.insert(7.0, elm0_rect);

        let elm0_rect = Rect::new(3, 3, 1, 1);
        let id2 = qt.insert(3.0, elm0_rect);

        let elm0_rect = Rect::new(-3, -3, 1, 1);
        let id3 = qt.insert(-3.0, elm0_rect);

        let elm0_rect = Rect::new(-6, -6, 1, 1);
        let id4 = qt.insert(-6.0, elm0_rect);

        //println!("\n\ntree:{:?}", qt);

        qt.remove(id0);

        assert_eq!(qt.nodes.active(), 9);


        // rmeove and cleanup
        qt.remove(id1);
        qt.remove(id3);
        qt.remove(id4);


        assert_eq!(qt.nodes.active(), 9);


        qt.cleanup();

        assert_eq!(qt.nodes.active(), 5);

        let elm0_rect = Rect::new(-100, -100, 1, 1);
        let id1 = qt.insert(7.0, elm0_rect);

        let elm0_rect = Rect::new(3, 3, 1, 1);
        let id2 = qt.insert(3.0, elm0_rect);

        let elm0_rect = Rect::new(-3, -3, 1, 1);
        let id3 = qt.insert(-3.0, elm0_rect);

        let elm0_rect = Rect::new(-6, -6, 1, 1);
        let id4 = qt.insert(-6.0, elm0_rect);

        assert_eq!(qt.nodes.active(), 9);



    }




    #[test]
    fn insert_4_elm() {

        let rect = Rect::new(-128, 128, 256, 256);

        let mut qt = QuadTree::<i32>::new(rect);

        qt.set_elements_per_node(6);

        // insert 6 top right elements
        for i in 0..6 {
            let elm_id = i;
            let elm_rect = Rect::new(10 + i, 10, 1, 1);
            qt.insert(elm_id, elm_rect);
        }


        // insert 6 top right elements
        for i in 0..6 {
            let elm_id = i + 10;
            let elm_rect = Rect::new(-10 - i, 10, 1, 1);
            qt.insert(elm_id, elm_rect);
        }


        // insert 6 bottom left elements
        for i in 0..6 {
            let elm_id = i + 20;
            let elm_rect = Rect::new(-10 - i, -10, 1, 1);
            qt.insert(elm_id, elm_rect);
        }


        // insert 6 bottom right elements
        for i in 0..6 {
            let elm_id = i + 30;
            let elm_rect = Rect::new(10 + i, -10, 1, 1);
            qt.insert(elm_id, elm_rect);
        }


        let points = qt.query_p(Point::new(15, 15));

        println!("{}", qt);
        assert_eq!(points.len(), 6);
        vec_compare(points, vec![0,1,2,3,4,5]);



        let points = qt.query_p(Point::new(-15, 15));

        assert_eq!(points.len(), 6);

        vec_compare(points, vec![10,11,12,13,14,15]);

    }

    #[test]
    fn insert_neg_50_50_elm() {

        let rect = Rect::from_points(Point {x: -128, y: -128}, Point { x: 128, y: 128} );

        let mut qt = QuadTree::<(i32, i32)>::new(rect);

        qt.set_elements_per_node(6);

        println!("{:?}", qt.elements_per_node);
        for i in (-51..49).step_by(2) {
            for j in (-51..49).step_by(2) {
                let rect = Rect::new(i,j,0,0);
                qt.insert((i,j), rect);
            }
        }

        let mut res = vec![];
        qt.query_p(Point::new(15, 15), -1 &mut res);

        assert_eq!(res), 1);

        let points0_0 = qt.query_p(Point::new(0, 0));
        assert_eq!(points0_0.len(), 16);

        let search_rect = Rect::from_points(Point {x: -10, y: -10}, Point { x: 10, y: 10} );
        let points = qt.query(&Query::rect(search_rect));

        assert_eq!(points.len(), 144)

}
*/
}
