use crate::bitset::ByteSet;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PointerByteNode {
    value: u64,
    children: [Option<Box<PointerByteNode>>; 256],
}

impl PointerByteNode {
    pub(crate) fn new(value: u64) -> Self {
        Self {
            value,
            children: [const { None }; 256],
        }
    }

    fn to_byteset(&self) -> ByteSet {
        let mut set = ByteSet::new();

        for (i, c) in self.children.iter().enumerate() {
            if c.is_some() {
                set.insert(i);
            }
        }

        set
    }

    #[cfg(test)]
    pub(crate) fn insert(&mut self, id: usize, value: u64) {
        assert!(self.children[id].is_none());
        self.children[id] = Some(Box::new(PointerByteNode::new(value)));
    }

    pub(crate) fn insert_child(&mut self, id: usize, value: Self) {
        self.children[id] = Some(Box::new(value));
    }

    #[cfg(test)]
    pub(crate) fn child_unwrap_mut(&mut self, id: usize) -> &mut Self {
        self.children[id].as_mut().unwrap()
    }

    #[cfg(test)]
    pub(crate) fn child_unwrap(&self, id: usize) -> &Self {
        self.children[id].as_ref().unwrap()
    }

    #[cfg(test)]
    pub(crate) fn value(&self) -> u64 {
        self.value
    }
}

fn append_children(curr_offset: usize, tree: &mut ByteTree, node: &PointerByteNode) {
    let num_children = node.num_children();
    tree.children_start.reserve(num_children);
    tree.nodes.reserve(num_children);

    let mut child_offset = curr_offset + num_children;
    for child in node.children.iter() {
        if let Some(subnode) = child {
            tree.nodes.push(subnode.to_byteset());
            tree.children_start.push(child_offset);
            tree.values.push(subnode.value);
            child_offset += subnode.tree_size() - 1;
        }
    }

    child_offset = curr_offset + num_children;
    for child in node.children.iter() {
        if let Some(subnode) = child {
            append_children(child_offset, tree, subnode);
            child_offset += subnode.tree_size() - 1;
        }
    }
}

impl PointerByteNode {
    fn num_children(&self) -> usize {
        self.children.iter().filter(|c| c.is_some()).count()
    }

    fn tree_size(&self) -> usize {
        let size_from_children: usize = self
            .children
            .iter()
            .map(|maybe_child| maybe_child.as_ref())
            .filter_map(|maybe_child| maybe_child.map(|c| c.tree_size()))
            .sum();
        size_from_children + 1
    }

    pub(crate) fn finalize(&self) -> ByteTree {
        let node_rep = self.to_byteset();

        let mut root_tree = ByteTree {
            values: vec![self.value],
            children_start: vec![1],
            nodes: vec![node_rep],
        };

        append_children(1, &mut root_tree, self);

        root_tree
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct ByteTree {
    values: Vec<u64>,
    children_start: Vec<usize>,
    nodes: Vec<ByteSet>,
}

impl ByteTree {
    pub(crate) fn walker(&self) -> ByteTreeWalker<'_> {
        ByteTreeWalker {
            tree: self,
            curr_offset: 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ByteTreeWalker<'a> {
    tree: &'a ByteTree,
    curr_offset: usize,
}

impl<'a> ByteTreeWalker<'a> {
    #[inline(always)]
    pub(crate) fn step(&self, id: usize) -> Option<Self> {
        if let Some(local_index) = self.tree.nodes[self.curr_offset].index(id) {
            let child_block_start = self.tree.children_start[self.curr_offset];
            Some(Self {
                tree: self.tree,
                curr_offset: child_block_start + (local_index as usize),
            })
        } else {
            None
        }
    }

    #[inline(always)]
    #[cfg(test)]
    pub(crate) fn curr_node(&self) -> &ByteSet {
        &self.tree.nodes[self.curr_offset]
    }

    #[inline(always)]
    pub(crate) fn value(&self) -> u64 {
        self.tree.values[self.curr_offset]
    }

    /// Should be the inverse of finalize
    #[cfg(test)]
    fn to_inefficient(self) -> PointerByteNode {
        let mut ptr_node = PointerByteNode::new(self.tree.values[self.curr_offset]);
        for i in 0..256 {
            if let Some(subwalker) = self.step(i) {
                ptr_node.children[i] = Some(Box::new(subwalker.to_inefficient()));
            }
        }
        ptr_node
    }
}

#[test]
fn test_finalize() {
    let mut tree = PointerByteNode::new(0);
    let empty_tree = tree.finalize();
    assert_eq!(empty_tree.children_start, vec![1]);
    assert_eq!(empty_tree.nodes, vec![ByteSet::new()]);

    // R
    //  -> 10
    tree.children[10] = Some(Box::new(PointerByteNode::new(1)));
    let single_tree = tree.finalize();
    assert_eq!(single_tree.children_start, vec![1, 2]);
    assert_eq!(
        single_tree.nodes,
        vec![ByteSet::from_entries([10]), ByteSet::new()]
    );

    // R
    //  -> 3
    //  -> 10
    tree.children[3] = Some(Box::new(PointerByteNode::new(2)));
    let double_tree = tree.finalize();
    assert_eq!(double_tree.children_start, vec![1, 3, 3]);
    assert_eq!(
        double_tree.nodes,
        vec![
            ByteSet::from_entries([3, 10]),
            ByteSet::new(),
            ByteSet::new(),
        ]
    );

    // R
    //  -> 3
    //      -> 5
    //  -> 10
    tree.children[3]
        .as_mut()
        .map(|n| n.children[5] = Some(Box::new(PointerByteNode::new(3))));
    let triple_tree = tree.finalize();
    assert_eq!(triple_tree.children_start, vec![1, 3, 4, 4]);
    assert_eq!(
        triple_tree.nodes,
        vec![
            ByteSet::from_entries([3, 10]),
            ByteSet::from_entries([5]),
            ByteSet::new(),
            ByteSet::new(),
        ]
    );

    // R
    //  -> 3
    //      -> 5
    //          -> 8
    //  -> 10
    tree.children[3].as_mut().unwrap().children[5]
        .as_mut()
        .unwrap()
        .children[8] = Some(Box::new(PointerByteNode::new(4)));
    let quad_tree = tree.finalize();
    assert_eq!(quad_tree.children_start, vec![1, 3, 5, 4, 5]);
    assert_eq!(
        quad_tree.nodes,
        vec![
            ByteSet::from_entries([3, 10]),
            ByteSet::from_entries([5]),
            ByteSet::new(),
            ByteSet::from_entries([8]),
            ByteSet::new(),
        ]
    );

    // R
    //  -> 3
    //      -> 5
    //          -> 8
    //  -> 10
    //      -> 4
    //      -> 6
    tree.children[10].as_mut().unwrap().children[4] = Some(Box::new(PointerByteNode::new(5)));
    tree.children[10].as_mut().unwrap().children[6] = Some(Box::new(PointerByteNode::new(6)));
    let tetra_tree = tree.finalize();
    assert_eq!(tetra_tree.children_start, vec![1, 3, 5, 4, 5, 7, 7]);
    assert_eq!(
        tetra_tree.nodes,
        vec![
            ByteSet::from_entries([3, 10]),
            ByteSet::from_entries([5]),
            ByteSet::from_entries([4, 6]),
            ByteSet::from_entries([8]),
            ByteSet::new(),
            ByteSet::new(),
            ByteSet::new(),
        ]
    )
}

#[test]
fn test_walker() {
    let mut tree = PointerByteNode::new(0);
    tree.insert(1, 10);
    tree.insert(5, 90);

    tree.child_unwrap_mut(1).insert(4, 1);
    tree.child_unwrap_mut(1).insert(100, 2);
    tree.child_unwrap_mut(1)
        .child_unwrap_mut(100)
        .insert(130, 3);

    tree.child_unwrap_mut(5).insert(30, 4);
    tree.child_unwrap_mut(5).child_unwrap_mut(30).insert(129, 5);

    let consolidated = tree.finalize();
    let walker = consolidated.walker();
    for i in 0..256 {
        if i != 1 && i != 5 {
            assert_eq!(walker.step(i), None);
        }
    }

    let w1 = walker.step(1).unwrap();

    for i in 0..256 {
        if i != 4 && i != 100 {
            assert_eq!(w1.step(i), None);
        }
    }
    let w1_4 = w1.step(4).unwrap();
    for i in 0..256 {
        assert_eq!(w1_4.step(i), None);
    }
    let w1_100 = w1.step(100).unwrap();
    for i in 0..256 {
        if i != 130 {
            assert_eq!(w1_100.step(i), None);
        }
    }
    let w1_100_130 = w1_100.step(130).unwrap();
    for i in 0..256 {
        assert_eq!(w1_100_130.step(i), None);
    }

    let mut walk = walker;
    for step in [5, 30, 129] {
        walk = walk.step(step).unwrap();
    }
    for i in 0..256 {
        assert_eq!(walk.step(i), None);
    }
}

#[test]
fn test_finalize_walk_inverse() {
    let mut tree = PointerByteNode::new(9);
    tree.insert(1, 8);
    tree.insert(5, 8);

    tree.child_unwrap_mut(1).insert(4, 6);
    tree.child_unwrap_mut(1).insert(100, 5);
    tree.child_unwrap_mut(1)
        .child_unwrap_mut(100)
        .insert(130, 4);

    tree.child_unwrap_mut(5).insert(30, 3);
    tree.child_unwrap_mut(5).child_unwrap_mut(30).insert(129, 2);

    assert_eq!(tree, tree.finalize().walker().to_inefficient());
}
