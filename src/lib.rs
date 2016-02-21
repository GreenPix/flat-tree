use std::ops::Deref;
use std::ops::DerefMut;
use std::iter::Zip;
use std::slice::{Iter, IterMut};
use std::mem;

use self::buffer::TreeNode;
use self::buffer::FlatTreeIter;
use self::buffer::FlatTreeIterMut;

pub mod buffer;

pub trait HasChildren: Sized {
    fn get_children(&self) -> &[Self];
}

#[derive(Debug)]
pub struct FlatTree<T> {
    buffer: Box<[TreeNode<T>]>,
}

impl<T> Deref for FlatTree<T> {
    type Target = [TreeNode<T>];

    fn deref<'a>(&'a self) -> &'a <Self as Deref>::Target {
        &self.buffer
    }
}

impl<T> DerefMut for FlatTree<T> {

    fn deref_mut<'a>(&'a mut self) -> &'a mut <Self as Deref>::Target {
        &mut self.buffer
    }
}

impl<T> FlatTree<T> {

    pub fn new<F, N>(root: &N, cap: usize, node_producer: F) -> FlatTree<T>
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        let mut buffer = Vec::with_capacity(cap);

        fill_buffer(
            &mut buffer,
            &mut None,
            &mut 0,
            root,
            &node_producer,
            true
        );

        FlatTree {
            buffer: buffer.into_boxed_slice(),
        }
    }

    pub fn node_as_index(&self, node: &TreeNode<T>) -> usize {
        let first = self.buffer.get(0).unwrap();
        assert!(node  as *const TreeNode<T> as usize >=
                first as *const TreeNode<T> as usize);
        let index = (node  as *const TreeNode<T> as usize -
                     first as *const TreeNode<T> as usize) /
            mem::size_of::<TreeNode<T>>();
        // If the diff is not in the range [0, len) then this is a bug.
        assert!((index) < self.buffer.len());
        // Return diff
        index
    }

    pub fn tree_iter<'a>(&'a self) -> FlatTreeIter<'a, T> {
        FlatTreeIter::new(&self.buffer)
    }

    pub fn tree_iter_mut<'a>(&'a mut self) -> FlatTreeIterMut<'a, T> {
        FlatTreeIterMut::new(&mut self.buffer)
    }

}

#[derive(Debug)]
pub struct FlatTreeLookup<T> {
    tree: FlatTree<T>,
    lookup_indices: Box<[usize]>,
}

impl<T> Deref for FlatTreeLookup<T> {
    type Target = FlatTree<T>;

    fn deref<'a>(&'a self) -> &'a <Self as Deref>::Target {
        &self.tree
    }
}

impl<T> DerefMut for FlatTreeLookup<T> {

    fn deref_mut<'a>(&'a mut self) -> &'a mut <Self as Deref>::Target {
        &mut self.tree
    }
}

impl <T> FlatTreeLookup<T> {
    pub fn new<F, N>(
        root: &N,
        cap: usize,
        node_producer: F) -> FlatTreeLookup<T>
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        let mut buffer = Vec::with_capacity(cap);
        let mut lookup_table = Some(Vec::with_capacity(cap));

        fill_buffer(
            &mut buffer,
            &mut lookup_table,
            &mut 0,
            root,
            &node_producer,
            true
        );

        FlatTreeLookup {
            tree: FlatTree{buffer: buffer.into_boxed_slice()},
            lookup_indices: lookup_table.unwrap().into_boxed_slice()
        }
    }

    pub fn enumerate_lookup_indices_mut<'a>(&'a mut self)
        -> Zip<Iter<usize>, IterMut<'a, TreeNode<T>>> {
        let FlatTreeLookup{ref mut tree, ref lookup_indices} = *self;
        let iter_mut = tree.iter_mut();
        lookup_indices.iter().zip(iter_mut)
    }

    pub fn enumerate_lookup_indices<'a>(&'a self)
        -> Zip<Iter<usize>, Iter<'a, TreeNode<T>>> {
        self.lookup_indices.iter().zip(self.buffer.iter())
    }

    /// Returns the index of the given node in the original tree.
    ///
    /// # Panics
    ///
    /// This method panics if the node given does not belong to this tree.
    pub fn node_as_global_index(&self, node: &TreeNode<T>) -> usize {
        let node_index = self.tree.node_as_index(node);
        self.lookup_indices[node_index]
    }
}


// ======================================== //
//                  HELPERS                 //
// ======================================== //

fn increment_index<N>(current_index: &mut usize, node: &N)
    where N: HasChildren
{
    *current_index += 1;

    for kid in node.get_children() {
        increment_index(current_index, kid);
    }
}

fn fill_buffer<F, N, T>(
    vec: &mut Vec<TreeNode<T>>,
    lookup_table: &mut Option<Vec<usize>>,
    current_index: &mut usize,
    node: &N,
    node_producer: &F,
    last_child: bool) -> isize
where N: HasChildren,
F: Fn(&N) -> Option<T>
{
    // Do we have a child here ?
    if let Some(new_child) = node_producer(node) {

        let index = vec.len();
        let mut next_sibling: isize = 1;
        let mut kids = node.get_children().len();

        // Default next sibling
        unsafe {
            // Las child with children.
            if kids > 0 {
                vec.push(TreeNode::new(new_child, -1));
                // Last child with no more children.
            } else {
                vec.push(TreeNode::new(new_child, 0));
            }
        }

        // Set values for lookup_table
        if let Some(ref mut lookup_indices) = lookup_table.as_mut() {
            lookup_indices.push(*current_index);
        }
        *current_index += 1;

        for kid in node.get_children() {
            kids -= 1;
            next_sibling += fill_buffer(
                vec,
                lookup_table,
                current_index,
                kid,
                node_producer.clone(),
                kids == 0
                );
        }

        if !last_child {
            unsafe {
                vec.get_unchecked_mut(index).set_next_sibling(next_sibling);
            }
        }

        next_sibling

    } else {
        // Child is ignored and all its sub tree.
        // Increment the index.
        increment_index(current_index, node);

        // Returns the next_sibling increment value
        0
    }
}
