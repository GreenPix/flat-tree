extern crate flat_tree;
use flat_tree::{HasChildren,FlatTreeIterMut};
use flat_tree::buffer::FlatTree;

#[derive(Debug)]
struct NonFlatNode {
    number: usize,
    children: Vec<NonFlatNode>,
}

impl NonFlatNode {
    fn new(number: usize) -> NonFlatNode {
        NonFlatNode {
            number: number,
            children: Vec::new(),
        }
    }

    fn with_children(number: usize, children: Vec<NonFlatNode>) -> NonFlatNode {
        NonFlatNode {
            number: number,
            children: children,
        }
    }
}

impl HasChildren for NonFlatNode {
    fn get_children(&self) -> &[NonFlatNode] {
        &self.children
    }
}

#[test]
fn test() {
    let test = NonFlatNode::with_children(1, vec![
                    NonFlatNode::new(2),
                    NonFlatNode::with_children(3, vec![
                        NonFlatNode::new(4),
                        NonFlatNode::new(5)]
                    ),
                    NonFlatNode::with_children(6, vec![
                        NonFlatNode::new(7)]
                    ),
                    NonFlatNode::new(8)]
                );
    let mut flat = FlatTree::new(&test, 8, |item| Some(item.number));
    let buffer: Vec<usize> = flat.iter().map(|x| **x).collect();
    assert_eq!(buffer, [1,2,3,4,5,6,7,8]);

    let mut res = Vec::new();
    recursive_iter(&mut res, flat.tree_iter_mut());
    assert_eq!(res,[">1", 
                        ">2",
                        "<2",
                        ">3",
                            ">4",
                            "<4",
                            ">5",
                            "<5",
                        "<3",
                        ">6",
                            ">7",
                            "<7",
                        "<6",
                        ">8",
                        "<8",
                    "<1"]);
}

fn recursive_iter(result: &mut Vec<String>, iter: FlatTreeIterMut<usize>) {
    for (node, mut children) in iter {
        result.push(format!(">{}", **node));
        recursive_iter(result, children.children_mut());
        result.push(format!("<{}", **node));
    }
}
