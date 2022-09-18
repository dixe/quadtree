use std::fmt;
mod free_list;
use free_list::FreeList;
use std::collections::VecDeque;

// From answer here: https://stackoverflow.com/questions/41946007/efficient-and-well-explained-implementation-of-a-quadtree-for-2d-collision-det


//QuadElt is stored once, and is referred to by QuadEltNode
#[derive(Debug)]
struct ElmRect {
    pub id: i32,
    pub rect: Rect
}

#[derive(Debug)]
struct ElmRectNode {
    //next node -1 is end of list
    pub next: i32,

    // index of element, both into elm_rects and into data
    pub elm_id: i32
}


impl ElmRectNode {

    pub fn insert(id: i32, node: &mut Node, element_nodes: &mut FreeList<ElmRectNode>) {

        //println!("Inserting node for element with id: {:?}", id);
        let elm_node_index = element_nodes.insert(ElmRectNode {
            next: -1,
            elm_id: id
        });

        println!("Insert = {:?}", (id, elm_node_index, &node));

        node.count += 1;

        let mut last_node_index = node.first_child;

        if last_node_index == -1  {
            println!("index = -1");
            node.first_child = elm_node_index;
        }
        else {

            while element_nodes[last_node_index].element.next != -1 {
                last_node_index = element_nodes[last_node_index].element.next;
            }
            println!("last_index = {}", last_node_index);
            element_nodes[last_node_index].element.next = elm_node_index;
        }
    }
}

#[derive(Debug)]
struct Node {
    // child are stored continiues
    // child0 (TL) = first_child
    // child1 (TR) = first_child + 1
    // child2 (BL) = first_child + 2
    // child3 (BR) = first_child + 3
    // if count is > 0 then its a leaf and first_child referres to element_nodes (ElmRectNode)
    pub first_child: i32,
    pub count: i32
}

impl Node {

    pub fn leaf() -> Self {
        Node {
            first_child: -1,
            count: 0
        }
    }

    pub fn iter_childs<T>(&self, qt: &QuadTree<T>) {

        for i in 0..self.count {
            println!("  ++  Iter: {:?}", (self.first_child + i, &qt.element_nodes[self.first_child + i].element));
        }
    }
}


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

#[derive(Debug, Clone)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32
}

#[derive(Debug)]
pub struct QuadPoint {
    pub x: i32,
    pub y: i32
}

impl Rect {

    pub fn from_points(p1: QuadPoint, p2: QuadPoint) -> Self {

        Rect {
            left: i32::min(p1.x, p2.x),
            right: i32::max(p1.x, p2.x),
            top:  i32::max(p1.y, p2.y),
            bottom: i32::min(p1.y, p2.y),
        }
    }

    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            left: x,
            right: x + w,
            top: y,
            bottom: y - h,
        }

    }

    fn location_quad(&self, i: usize) -> Rect {

        let node_middle_x = (self.right - self.left) / 2 + self.left;
        let node_middle_y = (self.top - self.bottom) / 2 + self.bottom;
        let middle_point = QuadPoint{x: node_middle_x, y: node_middle_y};
        return match i {
            // TL
            0 => Rect::from_points(middle_point, QuadPoint{x: self.left, y: self.top}),
            // TR
            1 => Rect::from_points(middle_point, QuadPoint{x: self.right, y: self.top}),
            // BL
            2 => Rect::from_points(middle_point, QuadPoint{x: self.left, y: self.bottom}),
            // BR
            3 => Rect::from_points(middle_point, QuadPoint{x: self.right, y: self.bottom}),
            _ => panic!("Location {} is not a valid location. Valid locations are: 0,1,2,3", i),
        }
    }


    fn point_quad_locations(node_rect: &Rect, point: &QuadPoint) -> [bool; 4] {

        // return bool for TL, TR, BL, BR

        let node_middle_x = (node_rect.right - node_rect.left) / 2 + node_rect.left;
        let node_middle_y = (node_rect.top - node_rect.bottom) / 2 + node_rect.bottom;

        //println!("({:?}, {})", node_middle_x, node_middle_y);
        // check is it inside on X and Y
        let tl = point.x <= node_middle_x &&
            point.x >= node_rect.left &&
            point.y >= node_middle_y &&
            point.y <= node_rect.top;


        let tr = point.x >= node_middle_x &&
            point.x <= node_rect.right &&
            point.y >= node_middle_y &&
            point.y <= node_rect.top;


        let bl = point.x <= node_middle_x &&
            point.x >= node_rect.left &&
            point.y <= node_middle_y &&
            point.y >= node_rect.bottom;


        let br = point.x >= node_middle_x &&
            point.x <= node_rect.right &&
            point.y <= node_middle_y &&
            point.y >= node_rect.bottom;

        //println!("\n\n{:#?} - {:#?}\n{:?}", node_rect, point,        [tl, tr, bl, br]);
        [tl, tr, bl, br]

    }

    fn element_quad_locations(node_rect: &Rect, element_rect: &Rect) -> [bool; 4] {

        // return bool for TL, TR, BL, BR

        let node_middle_x = (node_rect.right - node_rect.left) / 2 + node_rect.left;
        let node_middle_y = (node_rect.top - node_rect.bottom) / 2 + node_rect.bottom;


        // check is it inside on X and Y
        let tl = element_rect.left <= node_middle_x &&
            element_rect.right >= node_rect.left &&
            element_rect.top >= node_middle_y &&
            element_rect.bottom <= node_rect.top;


        let tr = element_rect.right >= node_middle_x &&
            element_rect.left <= node_rect.right &&
            element_rect.top >= node_middle_y &&
            element_rect.bottom <= node_rect.top;


        let bl = element_rect.left <= node_middle_x &&
            element_rect.right >= node_rect.left &&
            element_rect.bottom <= node_middle_y &&
            element_rect.top >= node_rect.bottom;


        let br = element_rect.right >= node_middle_x &&
            element_rect.left <= node_rect.right &&
            element_rect.bottom <= node_middle_y &&
            element_rect.top >= node_rect.bottom;


        [tl, tr, bl, br]

    }
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
    nodes: Vec::<Node>,

    // Actual data stored in three. Can be large structures
    data: Vec::<T>,

    // Rect for the root
    // All sub rects are computed on the fly in integers
    root_rect: Rect,

    max_depth: i32,
    nodes_per_cell: i32

}



// Public interface
impl<'a, T: std::fmt::Debug> QuadTree<T> {

    pub fn insert(&mut self, element: T, element_rect: Rect) ->  i32 {

        //println!("inserting {:?}", element_rect);
        // check if we can insert into root

        self.data.push(element);
        let data_id = (self.data.len() - 1) as i32;

        let element_id = self.elm_rects.insert(ElmRect {
            id: data_id,
            rect: element_rect.clone()
        });

        //println!("Inserting node for element with id: {:?}", element_id);
        let rect = self.root_rect.clone();
        self.insert_elm(element_id, 0, &element_rect, &rect, 0);
        element_id
    }

    pub fn get(&'a self, element_id: i32) -> &'a T {
        let data_id = self.elm_rects[element_id].element.id;

        &self.data[data_id as usize]
    }


    pub fn remove(&mut self, element_id: i32) {

        let elm = &self.elm_rects[element_id];

        println!("Removing {:?}", element_id);

        let leaves = self.find_leaves(&elm.element);

        for &leaf in &leaves {

            let leaf_node = &self.nodes[leaf as usize];
            println!("Processing leaf: {:?}", (leaf, leaf_node));

            leaf_node.iter_childs(self);

            for i in 0..leaf_node.count {

                let mut prev = -1;
                let mut cur = leaf_node.first_child + i;


                while cur != -1 {
                    let e =  &self.element_nodes[cur].element;
                    println!("lokking at: {:?}",(cur,e));
                    let next = e.next;
                    let elm_rect_id = e.elm_id;

                    if elm_rect_id == element_id {
                        if prev != -1 {
                            self.element_nodes[prev].element.next = next;
                        }

                        //self.element_nodes.erase(cur);
                        println!("FOUND ELEMENT TO REMOVE {}", cur);
                    } else {
                        prev = cur;
                    }

                    cur = next;
                }
            }
        }

        // also data?
        //self.elm_rects.erase(element_id);
    }

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


impl<'a, T: std::fmt::Debug> QuadTree<T> {

    pub fn new(rect: Rect) -> Self {

        let mut nodes = Vec::new();

        nodes.push(Node {
            first_child: -1,
            count: 0,
        });

        QuadTree {
            elm_rects: FreeList::new(),
            element_nodes: FreeList::new(),
            nodes,
            data: Vec::new(),
            root_rect: rect,
            max_depth: 10,
            nodes_per_cell: 2
        }
    }

    fn find_leaves(&self, elm_rect: & ElmRect) -> Vec::<i32> {
        let mut res = vec![];

        // start at root, at branches see which overlaps with elm.rect, process those too
        // return vec of nodes that elm.rect overlaps

        let mut to_process = VecDeque::new();

        let elms = self.elm_rects.range();

        //0 is root
        to_process.push_back(0);

        while let Some(node_id) = to_process.pop_front() {

            let node = &self.nodes[node_id];
            if node.count != -1 {
                res.push(node_id as i32);
            }
            else {
                let elm_node = &self.element_nodes[node_id as i32];
                let rect = &self.elm_rects[elm_node.element.elm_id].element.rect;
                let locations = Rect::element_quad_locations(&rect, &elm_rect.rect);
                println!(" LOCS {:?}", locations);

                for i in 0..4 {
                    if locations[i] {
                        let id = node.first_child as usize + i;
                        println!("Pushing :{:?}", id);
                        to_process.push_back(id);
                        // push node at this location
                    }
                }
            }

        }

        res
    }


    fn insert_elm(&mut self, element_id: i32,  node_index: usize, element_rect: &Rect, node_rect: &Rect, depth: i32) {


        //println!("node_index = {} depth = {} {:?}", node_index, depth, self.nodes[node_index]);

        // Check if leaf
        if self.nodes[node_index].count > -1 {
            // Check if we can just insert into this node
            if self.nodes[node_index].count < self.nodes_per_cell  || depth >= self.max_depth {
                //println!("insert into leaf");
                ElmRectNode::insert(element_id, &mut self.nodes[node_index], &mut self.element_nodes);
            }
            // make this into not a leaf, but a branch
            else {
                //println!("branching");
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


    fn insert_into_branch(&mut self, element_id: i32, node_index: usize, element_rect: &Rect, node_rect: &Rect, depth: i32) {

        // We are at a branch
        // check which children it should be se into
        let locations = Rect::element_quad_locations(node_rect, element_rect);


        for i in 0..4 {
            if locations[i] {
                let new_rect = node_rect.location_quad(i);

                let new_node_index = (self.nodes[node_index].first_child as usize) + i;

                self.insert_elm(element_id, new_node_index, element_rect, &new_rect, depth + 1);
            }
        }
    }



    fn split(&mut self, node_index: usize, node_rect: &Rect) {
        //println!("Making leaf into branch {:?}", node_index);

        self.nodes.push(Node::leaf());

        let new_first_child = self.nodes.len() - 1;

        self.nodes.push(Node::leaf());
        self.nodes.push(Node::leaf());
        self.nodes.push(Node::leaf());


        let mut next_child = self.nodes[node_index].first_child;

        while next_child != -1 {


            //println!("Reallocate element {:?}", self.element_nodes[next_child].element);
            //println!("Original child count {}", self.nodes[node_index].count );
            let reallocated_id = self.element_nodes[next_child].element.elm_id;

            let new_next_child = self.element_nodes[next_child].element.next;

            self.element_nodes.erase(next_child);

            let child_rect = &self.elm_rects[reallocated_id].element.rect;
            let locations = Rect::element_quad_locations(node_rect, child_rect);



            for i in 0..4 {
                if locations[i] {
                    ElmRectNode::insert(reallocated_id, &mut self.nodes[new_first_child + i], &mut self.element_nodes);
                }
            }

            next_child = new_next_child;

        }



        // set first child as the first quadnode TL
        // and set count to -1 to indicate it is a branch
        self.nodes[node_index].first_child = new_first_child as i32;
        self.nodes[node_index].count = -1;
    }




    fn query_node_box(&self, node_index: usize, node_rect: &Rect, query: &Query, data_vec: &mut std::collections::HashSet::<i32>) {
        // leaf, return  all elements
        if self.nodes[node_index].count > -1 {
            let mut child_index = self.nodes[node_index].first_child;

            while child_index != -1 {
                data_vec.insert(self.elm_rects[self.element_nodes[child_index].element.elm_id].element.id);

                child_index = self.element_nodes[child_index].element.next;

            }
        }
        else {
            self.query_branch(node_index, node_rect, query, data_vec);
        }
    }


    fn query_branch(&self, node_index: usize, node_rect: &Rect, query: &Query, data_vec: &mut std::collections::HashSet::<i32>) {

        let locations = match query {
            Query::Point(p) => Rect::point_quad_locations(node_rect, p),
            Query::Rect(r) => Rect::element_quad_locations(node_rect, r )
        };

        for i in 0..4 {
            if locations[i] {
                // point is inside this rect
                self.query_node_box((self.nodes[node_index].first_child as usize) + i, &node_rect.location_quad(i), query, data_vec);
            }
        }
    }



    fn print(&self) -> String {
        self.print_node(0, 0)
    }

    fn print_node(&self, node_index: usize, indent: usize) -> String {

        if self.nodes[node_index].count >= 0 {
            // leaf

            if self.nodes[node_index].count > 0 {
                let mut child_index = self.nodes[node_index].first_child;

                let mut res = "".to_string();
                while child_index != -1 {
                    let elm_node = &self.element_nodes[child_index].element;
                    res += &format!(" element({}): {:?}, node: {:?} | ", elm_node.elm_id, self.data[elm_node.elm_id as usize], child_index);
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
            let first_index = self.nodes[node_index].first_child as usize;

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

        println!("{:?}", locations);

        assert!(locations[0] && locations[1] && locations[2] && locations[3]);

    }



    #[test]
    fn node_locations_tl() {

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: -10, y: 10}, QuadPoint { x: -20, y: 20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        println!("{:?}", locations);

        assert!(locations[0] && !locations[1] && !locations[2] && !locations[3]);

    }


    #[test]
    fn node_locations_tr() {

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: 10, y: 10}, QuadPoint { x: 20, y: 20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        println!("{:?}", locations);

        assert!(!locations[0] && locations[1] && !locations[2] && !locations[3]);
    }


    #[test]
    fn node_locations_bl() {

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: -10, y: -10}, QuadPoint { x: -20, y: -20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        println!("{:?}", locations);

        assert!(!locations[0] && !locations[1] && locations[2] && !locations[3]);
    }



    #[test]
    fn node_locations_br() {

        let node_rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let element_rect = Rect::from_points(QuadPoint {x: 10, y: -10}, QuadPoint { x: 20, y: -20} );

        let locations = Rect::element_quad_locations(&node_rect, &element_rect);

        println!("{:?}", locations);

        assert!(!locations[0] && !locations[1] && !locations[2] && locations[3]);
    }

    #[test]
    fn insert_2_elm() {


        let rect = Rect::from_points(QuadPoint {x: -128, y: -128}, QuadPoint { x: 128, y: 128} );

        let mut qt = QuadTree::<f32>::new(rect);


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


        assert_eq!(points0.len(), 2);
        vec_compare(points0, vec![1.0, 2.0]);


        let points1 = qt.query(&Query::point(-1,-1));
        assert_eq!(points1.len(), 2);
        vec_compare(points1, vec![1.0, 2.0]);

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

        println!("tree:{:?}", qt);


        qt.remove(id00);


        println!("tree:{:?}", qt);
        assert!(false);



    }

   #[test]
    fn insert_remove_2() {

        let rect = Rect::new(-128, 128, 256, 256);

        let mut qt = QuadTree::<f32>::new(rect);

        let elm0_rect = Rect::new(5, 5, 1, 1);
        let id0 = qt.insert(5.0, elm0_rect);

        let elm0_rect = Rect::new(7, 7, 1, 1);
        let id0 = qt.insert(7.0, elm0_rect);

        let elm0_rect = Rect::new(9,9, 1, 1);
        let id0 = qt.insert(9.0, elm0_rect);

        let elm0_rect = Rect::new(13,13, 1, 1);
        let id0 = qt.insert(13.0, elm0_rect);

        //println!("{:?}", id0);
        println!("tree:{:?}", qt);

        qt.remove(0);

        println!("tree:{:?}", qt);
        assert!(false);



    }




    #[test]
    fn insert_4_elm() {

        let rect = Rect::new(-128, 128, 256, 256);

        let mut qt = QuadTree::<i32>::new(rect);


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


        println!("{:#?}", qt);



        let points = qt.query(&Query::point(15, 15));

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


        for i in (-51..49).step_by(2) {
            for j in (-51..49).step_by(2) {
                let rect = Rect::new(i,j,0,0);
                qt.insert((i,j), rect);
            }
        }

        let points15_15 = qt.query(&Query::point(15, 15));

        assert_eq!(points15_15.len(), 4);

        let points0_0 = qt.query(&Query::point(0, 0));
        assert_eq!(points0_0.len(), 16);

        let search_rect = Rect::from_points(QuadPoint {x: -10, y: -10}, QuadPoint { x: 10, y: 10} );
        let points = qt.query(&Query::rect(search_rect));

        assert_eq!(points.len(), 144)

    }
}
