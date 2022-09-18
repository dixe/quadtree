pub struct QuadTree<T> {

    // All quads of elements in the quadtree
    elements: FreeList<QuadElt>,

    // All elementNodes in quadTree
    // Elements nodes refer to elements
    element_nodes: FreeList<QuadEltNode>,


    // All nodes in quadTree
    // First node is the root
    // leafs are where count > 0 and then first child is index into element_nodes
    nodes: Vec::<QuadNode>,

    // Actual data stored in three. Can be large structures
    data: Vec::<T>,

    // Rect for the root
    // All sub rects are computed on the fly in integers
    root_rect: QuadRect,

    max_depth: i32,
    nodes_per_cell: i32

}

impl<T> QuadTree<T> {



}
