#![feature(nonzero)]
extern crate core;
use core::nonzero::NonZero;
use std::ops::Deref;
use std::ops::DerefMut;
use std::mem;
use std::marker::PhantomData;

pub mod buffer;

pub trait HasChildren: Sized {
    fn get_children(&self) -> &[Self];
}

// Next sibling meaning:
// * -1  : No sibling, but some children
// * 0   : No sibling and no children
// * 1   : siblings, but no children
// * > 1 : siblings and children

#[derive(Debug)]
pub struct TreeNode<T> {
    data: T,
    next_sibling: isize,
}

impl<T> Deref for TreeNode<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        &self.data
    }
}

impl<T> DerefMut for TreeNode<T> {

    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.data
    }
}

impl<T> TreeNode<T> {

    unsafe fn new(data: T, next_sibling: isize) -> TreeNode<T> {
        TreeNode {
            data: data,
            next_sibling: next_sibling,
        }
    }

    #[inline]
    unsafe fn set_next_sibling(&mut self, next_sibling: isize) {
        self.next_sibling = next_sibling;
    }
}

/// Const iterator over FlatTree
pub struct FlatTreeIter<'a, T: 'a> {
    current: Option<NonZero<*const TreeNode<T>>>,
    _marker: PhantomData<&'a TreeNode<T>>,
}

impl<'a, T> FlatTreeIter<'a, T> {
    pub fn new(flat: &'a [TreeNode<T>]) -> FlatTreeIter<'a, T> {
        unsafe {
            FlatTreeIter {
                current: flat.first().map(|pointer| NonZero::new(pointer as *const TreeNode<T>)),
                _marker: PhantomData,
            }
        }
    }

    pub fn new_empty() -> FlatTreeIter<'a, T> {
        FlatTreeIter {
            current: None,
            _marker: PhantomData
        }
    }
}

impl<'a, T: 'a> Iterator for FlatTreeIter<'a, T> {
    type Item = (&'a TreeNode<T>, Children<'a, T>);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.current.take().map(|ptr|{
            unsafe {
                let node: &TreeNode<T> = mem::transmute(ptr);
                if node.next_sibling > 0 {
                    self.current = Some(NonZero::new(ptr.offset(node.next_sibling)));
                }
                let children = Children::new(node);
                (mem::transmute(node), children)
            }
        })
    }
}

/// Mutable iterator over FlatTree
pub struct FlatTreeIterMut<'a, T: 'a> {
    current: Option<NonZero<*mut TreeNode<T>>>,
    _marker: PhantomData<&'a mut TreeNode<T>>,
}

impl<'a, T> FlatTreeIterMut<'a, T> {
    pub fn new(flat: &'a mut [TreeNode<T>]) -> FlatTreeIterMut<'a, T> {
        unsafe {
            FlatTreeIterMut {
                current: flat.first_mut().map(|pointer| NonZero::new(pointer as *mut TreeNode<T>)),
                _marker: PhantomData,
            }
        }
    }

    pub fn new_empty() -> FlatTreeIterMut<'a, T> {
        FlatTreeIterMut {
            current: None,
            _marker: PhantomData
        }
    }
}

impl<'a, T: 'a> Iterator for FlatTreeIterMut<'a, T> {
    type Item = (&'a mut TreeNode<T>, ChildrenMut<'a, T>);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.current.take().map(|ptr|{
            unsafe {
                let node: &mut TreeNode<T> = mem::transmute(ptr);
                if node.next_sibling > 0 {
                    self.current = Some(NonZero::new(ptr.offset(node.next_sibling)));
                }
                let children = ChildrenMut::new(node);
                (mem::transmute(node), children)
            }
        })
    }
}

pub struct ChildrenMut<'a, T: 'a> {
    _marker: PhantomData<&'a mut TreeNode<T>>,
    parent: NonZero<*mut TreeNode<T>>,
}

impl <'a, T> ChildrenMut<'a, T> {
    pub fn children_mut<'b>(&'b mut self) -> FlatTreeIterMut<'b, T> {
        unsafe {
            let pointer = if (**self.parent).next_sibling > 1 || (**self.parent).next_sibling == -1 {
                Some(NonZero::new(self.parent.offset(1)))
            } else {
                None
            };
            FlatTreeIterMut {
                _marker: PhantomData,
                current: pointer,
            }
        }
    }

    pub fn get_mut<'b>(&'b mut self, index: usize) -> Option<(&'b mut TreeNode<T>, ChildrenMut<'b, T>)> {
        self.children_mut().nth(index)
    }

    pub fn children<'b>(&'b self) -> FlatTreeIter<'b, T> {
        unsafe {
            let pointer = if (**self.parent).next_sibling > 1 || (**self.parent).next_sibling == -1 {
                Some(NonZero::new(self.parent.offset(1) as *const TreeNode<T>))
            } else {
                None
            };
            FlatTreeIter {
                _marker: PhantomData,
                current: pointer,
            }
        }
    }

    pub fn get<'b>(&'b self, index: usize) -> Option<(&'b TreeNode<T>, Children<'b, T>)> {
        self.children().nth(index)
    }

    pub fn is_empty(&self) -> bool {
        unsafe {
            if (**self.parent).next_sibling > 1 || (**self.parent).next_sibling == -1 {
                false
            } else {
                true
            }
        }
    }

    fn new(node: *mut TreeNode<T>) -> ChildrenMut<'a, T> {
        unsafe {
            ChildrenMut {
                parent: NonZero::new(node),
                _marker: PhantomData,
            }
        }
    }
}

pub struct Children<'a, T: 'a> {
    _marker: PhantomData<&'a TreeNode<T>>,
    parent: NonZero<*const TreeNode<T>>,
}

impl <'a, T> Children<'a, T> {
    pub fn children<'b>(&'b self) -> FlatTreeIter<'b, T> {
        unsafe {
            let pointer = if (**self.parent).next_sibling > 1 || (**self.parent).next_sibling == -1 {
                Some(NonZero::new(self.parent.offset(1)))
            } else {
                None
            };
            FlatTreeIter {
                _marker: PhantomData,
                current: pointer,
            }
        }
    }

    pub fn get<'b>(&'b self, index: usize) -> Option<(&'b TreeNode<T>, Children<'b, T>)> {
        self.children().nth(index)
    }

    pub fn is_empty(&self) -> bool {
        unsafe {
            if (**self.parent).next_sibling > 1 || (**self.parent).next_sibling == -1 {
                false
            } else {
                true
            }
        }
    }

    fn new(node: *const TreeNode<T>) -> Children<'a, T> {
        unsafe {
            Children {
                parent: NonZero::new(node),
                _marker: PhantomData,
            }
        }
    }
}
