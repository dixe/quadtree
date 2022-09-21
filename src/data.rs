use crate::free_list::FreeList;

//QuadElt is stored once, and is referred to by QuadEltNode
#[derive(Debug)]
pub(crate) struct ElmRect {
    pub data_id: i32,
    pub rect: Rect
}

#[derive(Debug)]
pub(crate) struct ElmRectNode {
    //next node -1 is end of list
    pub next: i32,

    // index of element, both into elm_rects and into data
    pub elm_id: i32
}


impl ElmRectNode {

    pub fn insert(id: i32, node: &mut Node, element_nodes: &mut FreeList<ElmRectNode>) {

        let current_first_child = node.first_child;

        //println!("Inserting node for element with id: {:?}", id);
        let elm_node_index = element_nodes.insert(ElmRectNode {
            next: current_first_child,
            elm_id: id
        });

        node.count += 1;

        node.first_child = elm_node_index;
    }
}

#[derive(Debug)]
pub(crate) struct Node {
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

    pub fn is_branch(&self) -> bool {
        self.count == -1
    }

    pub fn is_leaf(&self) -> bool {
        // can be 0 if all elements have been removed
        self.count >= 0
    }
}



#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

impl Rect {

    pub fn from_points(p1: Point, p2: Point) -> Self {

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

    pub(crate) fn location_quads(&self) -> [Rect; 4] {

        let node_middle_x = (self.right - self.left) / 2 + self.left;
        let node_middle_y = (self.top - self.bottom) / 2 + self.bottom;
        let middle_point = Point{x: node_middle_x, y: node_middle_y};

        [Rect::from_points(middle_point, Point{x: self.left, y: self.top}),
         Rect::from_points(middle_point, Point{x: self.right, y: self.top}),
         Rect::from_points(middle_point, Point{x: self.left, y: self.bottom}),
         Rect::from_points(middle_point, Point{x: self.right, y: self.bottom}),
        ]
    }

    pub fn intersect(&self, other: Rect) -> bool {
        self.left <= other.right &&
            self.right >= other.left &&
            self.top >= other.bottom &&
            self.bottom <= other.top

    }

     pub(crate) fn point_quad_locations(node_rect: Rect, point: &Point) -> [bool; 4] {

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

        [tl, tr, bl, br]

    }

     pub(crate) fn element_quad_locations(node_rect: Rect, element_rect: Rect) -> [bool; 4] {

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
