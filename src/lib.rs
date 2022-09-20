use std::fmt;
mod free_list;
use free_list::FreeList;
use std::collections::VecDeque;

mod query;
mod data;

pub use self::data::*;
pub use self::query::*;
// From answer here: https://stackoverflow.com/questions/41946007/efficient-and-well-explained-implementation-of-a-quadtree-for-2d-collision-det

use data::*;
use query::*;

struct FindLeaves {
    node_id: i32,
    rect: Rect
}

pub struct QuadTree<T> {

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
    elements_per_node: i32

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
            elements_per_node: 2
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
        let rect = self.root_rect.clone();
        self.insert_elm(element_id, 0, &element_rect, &rect, 0);
        element_id
    }

    pub fn set_elements_per_node(&mut self, npc: i32) {
        self.elements_per_node = i32::max(1, npc);
    }

    /// Removes an element from the tree. Does not restructure the tree see ['cleanup()']
    pub fn remove(&mut self, element_id: i32) {

        let elm = &self.elm_rects[element_id];
        let leaves = self.find_leaves(&elm);
        for &leaf in &leaves {

            let leaf_node = &mut self.nodes[leaf];


            let first_child = leaf_node.first_child;
            for i in 0..leaf_node.count {

                let mut prev = -1;
                let mut cur = first_child + i;

                while cur != -1 {
                    let e =  &self.element_nodes[cur];

                    let next = e.next;
                    let elm_rect_id = e.elm_id;

                    if elm_rect_id == element_id {
                        leaf_node.count -= 1;
                        if prev != -1 { //  in the middle of element chain
                            self.element_nodes[prev].next = next;
                        }
                        else { // head of element chain, change the leaf node
                            leaf_node.first_child = self.element_nodes[cur].next;
                            //self.nodes[leaf_node]
                        }

                        self.element_nodes.erase(cur);
                    } else {
                        prev = cur;
                    }

                    cur = next;
                }
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

}


// Private functions
impl<'a, T: std::fmt::Debug> QuadTree<T> {



    fn find_leaves(&self, elm_rect: & ElmRect) -> Vec::<i32> {
        let mut res = vec![];

        // start at root, at branches see which overlaps with elm.rect, process those too
        // return vec of nodes that elm.rect overlaps

        let mut to_process = VecDeque::new();

        //0 is root
        to_process.push_back(FindLeaves{ node_id:0, rect: self.root_rect});


        while let Some(node_data) = to_process.pop_front() {


            let node = &self.nodes[node_data.node_id];

            // if node is a leaf, push to result
            if node.count != -1 {
                res.push(node_data.node_id as i32);
            }
            else {

                // is a branch, see which child quads elements fits into
                let locations = Rect::element_quad_locations(&node_data.rect, &elm_rect.rect);

                for i in 0..4 {
                    if locations[i] {
                        // add matching child to process_list and calc the quad
                        let new_rect = node_data.rect.location_quad(i);
                        to_process.push_back(FindLeaves {node_id: node.first_child + i as i32, rect: new_rect});
                    }
                }
            }

        }

        res
    }


    fn insert_elm(&mut self, element_id: i32,  node_index: i32, element_rect: &Rect, node_rect: &Rect, depth: i32) {


        //println!("node_index = {} depth = {} {:?}", node_index, depth, self.nodes[node_index]);

        // Check if leaf
        if self.nodes[node_index].count > -1 {
            // Check if we can just insert into this node
            if self.nodes[node_index].count < self.elements_per_node || depth >= self.max_depth {
                //println!("insert into leaf");
                ElmRectNode::insert(element_id, &mut self.nodes[node_index], &mut self.element_nodes);
            }
            // make this into not a leaf, but a branch
            else {
                self.split(node_index, node_rect);

                self.nodes[node_index].count = -1;

                self.insert_into_branch(element_id, node_index, element_rect, node_rect, depth);
            }

        }
        else {
            //println!("insert into branch");
            self.insert_into_branch(element_id, node_index, element_rect, node_rect, depth);
        }
    }


    fn insert_into_branch(&mut self, element_id: i32, node_index: i32, element_rect: &Rect, node_rect: &Rect, depth: i32) {

        // We are at a branch
        // check which children it should be se into
        let locations = Rect::element_quad_locations(node_rect, element_rect);


        for i in 0..4 {
            if locations[i] {
                let new_rect = node_rect.location_quad(i);

                let new_node_index = (self.nodes[node_index].first_child) + i as i32;

                self.insert_elm(element_id, new_node_index, element_rect, &new_rect, depth + 1);
            }
        }
    }



    fn split(&mut self, node_index: i32, node_rect: &Rect) {
        //println!("Making leaf into branch {:?}", node_index);

        let index = self.nodes.insert(Node::leaf());
        let index2 = self.nodes.insert(Node::leaf());
        let index3 = self.nodes.insert(Node::leaf());
        let index4 = self.nodes.insert(Node::leaf());


        let new_first_child = index;


        assert!(index == index2 - 1 && index2 == index3 - 1 && index3 == index4 - 1);


        let mut next_child = self.nodes[node_index].first_child;

        while next_child != -1 {


            //println!("Reallocate element {:?}", self.element_nodes[next_child].element);
            //println!("Original child count {}", self.nodes[node_index].count );
            let reallocated_id = self.element_nodes[next_child].elm_id;

            let new_next_child = self.element_nodes[next_child].next;

            self.element_nodes.erase(next_child);

            let child_rect = &self.elm_rects[reallocated_id].rect;
            let locations = Rect::element_quad_locations(node_rect, child_rect);

            for i in 0..4 {
                if locations[i] {
                    ElmRectNode::insert(reallocated_id, &mut self.nodes[new_first_child + i as i32], &mut self.element_nodes);
                }
            }

            next_child = new_next_child;

        }


        // set first child as the first quadnode TL
        // and set count to -1 to indicate it is a branch
        self.nodes[node_index].first_child = new_first_child as i32;
        self.nodes[node_index].count = -1;
    }


    fn query_node_box(&self, node_index: i32, node_rect: &Rect, query: &Query, data_vec: &mut  Vec::<i32>) {
        // leaf, return  all elements
        if self.nodes[node_index].count > -1 {
            let mut child_index = self.nodes[node_index].first_child;

            while child_index != -1 {
                data_vec.push(self.elm_rects[self.element_nodes[child_index].elm_id].data_id);
                child_index = self.element_nodes[child_index].next;

            }
        }
        else {
            self.query_branch(node_index, node_rect, query, data_vec);
        }
    }


    fn query_branch(&self, node_index: i32, node_rect: &Rect, query: &Query, data_vec: &mut Vec::<i32>) {

        let locations = match query {
            Query::Point(p) => Rect::point_quad_locations(node_rect, p),
            Query::Rect(r) => Rect::element_quad_locations(node_rect, r )
        };

        for i in 0..4 {
            if locations[i] {
                // point is inside this rect
                self.query_node_box((self.nodes[node_index].first_child) + i as i32, &node_rect.location_quad(i), query, data_vec);
            }
        }
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
                    res += &format!(" e({}): {:?}, idx: {:?} | ", elm_node.elm_id, self.data[elm_node.elm_id], child_index);
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

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: -10, y: -10}, QuadPoint { x: 20, y: 20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        //println!("{:?}", locations);

        assert!(locations[0] && locations[1] && locations[2] && locations[3]);

    }



    #[test]
    fn node_locations_tl() {

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: -10, y: 10}, QuadPoint { x: -20, y: 20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        //println!("{:?}", locations);

        assert!(locations[0] && !locations[1] && !locations[2] && !locations[3]);

    }


    #[test]
    fn node_locations_tr() {

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: 10, y: 10}, QuadPoint { x: 20, y: 20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        //println!("{:?}", locations);

        assert!(!locations[0] && locations[1] && !locations[2] && !locations[3]);
    }


    #[test]
    fn node_locations_bl() {

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: -10, y: -10}, QuadPoint { x: -20, y: -20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        //println!("{:?}", locations);

        assert!(!locations[0] && !locations[1] && locations[2] && !locations[3]);
    }



    #[test]
    fn node_locations_br() {

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: 10, y: -10}, QuadPoint { x: 20, y: -20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        //println!("{:?}", locations);

        assert!(!locations[0] && !locations[1] && !locations[2] && locations[3]);
    }

    #[test]
    fn insert_2_elm() {


        let rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let mut qt = QuadTree::<f32>::new(rect);

        qt.set_elements_per_node(6);

        let elm1_id = 1.0;
        let elm1_rect = Rect::from_points(QuadPoint {x: 10, y: 10}, QuadPoint { x: 20, y: 20} );
        qt.insert(elm1_id, elm1_rect);

        let elm2_id = 2.0;
        let elm2_rect = Rect::from_points(QuadPoint {x: -10, y: -10}, QuadPoint { x: -20, y: -20} );
        qt.insert(elm2_id, elm2_rect);

        let elm2_id = 3.0;
        let elm2_rect = Rect::from_points(QuadPoint {x: 0, y: 0}, QuadPoint { x: 5, y: 5} );
        qt.insert(elm2_id, elm2_rect);


        let points0 = qt.query(&Query::point(15,15));


        assert_eq!(points0.len(), 3);
        vec_compare(points0, vec![1.0, 2.0, 3.0]);


        let points1 = qt.query(&Query::point(-1,-1));
        assert_eq!(points1.len(), 3);
        vec_compare(points1, vec![1.0, 2.0, 3.0]);

    }

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


        let points = qt.query(&Query::point(15, 15));

        println!("{}", qt);
        assert_eq!(points.len(), 6);
        vec_compare(points, vec![0,1,2,3,4,5]);



        let points = qt.query(&Query::point(-15, 15));

        assert_eq!(points.len(), 6);

        vec_compare(points, vec![10,11,12,13,14,15]);

    }

    fn vec_compare<T>(res: Vec::<&T>, expected: Vec::<T>) where T: fmt::Debug + Copy + PartialOrd {
        let mut values = Vec::<T>::new();

        for p in &res {
            values.push(**p);
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(expected, values);
    }

    #[test]
    fn insert_neg_50_50_elm() {

        let rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let mut qt = QuadTree::<(i32, i32)>::new(rect);

        qt.set_elements_per_node(6);

        println!("{:?}", qt.elements_per_node);
        for i in (-51..49).step_by(2) {
            for j in (-51..49).step_by(2) {
                let rect = Rect::new(i,j,0,0);
                qt.insert((i,j), rect);
            }
        }

        let points15_15 = qt.query(&Query::point(15, 15));

        println!("{:?}", points15_15);
        assert_eq!(points15_15.len(), 4);

        let points0_0 = qt.query(&Query::point(0, 0));
        assert_eq!(points0_0.len(), 16);

        let search_rect = Rect::from_points(QuadPoint {x: -10, y: -10}, QuadPoint { x: 10, y: 10} );
        let points = qt.query(&Query::rect(search_rect));

        assert_eq!(points.len(), 144)

    }
}
