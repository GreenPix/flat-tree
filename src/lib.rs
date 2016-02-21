use std::ops::Deref;
use std::ops::DerefMut;
use std::mem;
use std::marker::PhantomData;
use std::ptr;

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
    current: *const TreeNode<T>,
    _marker: PhantomData<&'a TreeNode<T>>,
}

impl<'a, T> FlatTreeIter<'a, T> {
    pub fn new(flat: &'a [TreeNode<T>]) -> FlatTreeIter<'a, T> {
        FlatTreeIter {
            current: flat.first().map(|p| p as *const TreeNode<T>).unwrap_or(ptr::null()),
            _marker: PhantomData,
        }
    }

    pub fn new_empty() -> FlatTreeIter<'a, T> {
        FlatTreeIter {
            current: ptr::null(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for FlatTreeIter<'a, T> {
    type Item = (&'a TreeNode<T>, Children<'a, T>);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.current.is_null() {
            None
        } else {
            unsafe {
                let node: &TreeNode<T> = mem::transmute(self.current);
                if node.next_sibling > 0 {
                    self.current = self.current.offset(node.next_sibling);
                } else {
                    self.current = ptr::null();
                }
                let children = Children::new(node);
                Some((mem::transmute(node), children))
            }
        }
    }
}

/// Mutable iterator over FlatTree
pub struct FlatTreeIterMut<'a, T: 'a> {
    current: *mut TreeNode<T>,
    _marker: PhantomData<&'a mut TreeNode<T>>,
}

impl<'a, T> FlatTreeIterMut<'a, T> {
    pub fn new(flat: &'a mut [TreeNode<T>]) -> FlatTreeIterMut<'a, T> {
        FlatTreeIterMut {
            current: flat.first_mut().map(|p| p as *mut TreeNode<T>).unwrap_or(ptr::null_mut()),
            _marker: PhantomData,
        }
    }

    pub fn new_empty() -> FlatTreeIterMut<'a, T> {
        FlatTreeIterMut {
            current: ptr::null_mut(),
            _marker: PhantomData
        }
    }
}

impl<'a, T: 'a> Iterator for FlatTreeIterMut<'a, T> {
    type Item = (&'a mut TreeNode<T>, ChildrenMut<'a, T>);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.current.is_null() {
            None
        } else {
            unsafe {
                let node: &mut TreeNode<T> = mem::transmute(self.current);
                if node.next_sibling > 0 {
                    self.current = self.current.offset(node.next_sibling);
                } else {
                    self.current = ptr::null_mut();
                }
                let children = ChildrenMut::new(node);
                Some((node, children))
            }
        }
    }
}

pub struct ChildrenMut<'a, T: 'a> {
    _marker: PhantomData<&'a mut TreeNode<T>>,
    parent: *mut TreeNode<T>,
}

impl <'a, T> ChildrenMut<'a, T> {
    pub fn children_mut<'b>(&'b mut self) -> FlatTreeIterMut<'b, T> {
        unsafe {
            let pointer = if (*self.parent).next_sibling > 1 || (*self.parent).next_sibling == -1 {
                self.parent.offset(1)
            } else {
                ptr::null_mut()
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
            let pointer = if (*self.parent).next_sibling > 1 || (*self.parent).next_sibling == -1 {
                self.parent.offset(1) as *const TreeNode<T>
            } else {
                ptr::null()
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
            if (*self.parent).next_sibling > 1 || (*self.parent).next_sibling == -1 {
                false
            } else {
                true
            }
        }
    }

    fn new(node: *mut TreeNode<T>) -> ChildrenMut<'a, T> {
        ChildrenMut {
            parent: node,
            _marker: PhantomData,
        }
    }
}

pub struct Children<'a, T: 'a> {
    _marker: PhantomData<&'a TreeNode<T>>,
    parent: *const TreeNode<T>,
}

impl <'a, T> Children<'a, T> {
    pub fn children<'b>(&'b self) -> FlatTreeIter<'b, T> {
        unsafe {
            let pointer = if (*self.parent).next_sibling > 1 || (*self.parent).next_sibling == -1 {
                self.parent.offset(1)
            } else {
                ptr::null()
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
            if (*self.parent).next_sibling > 1 || (*self.parent).next_sibling == -1 {
                false
            } else {
                true
            }
        }
    }

    fn new(node: *const TreeNode<T>) -> Children<'a, T> {
        Children {
            parent: node,
            _marker: PhantomData,
        }
    }
}
