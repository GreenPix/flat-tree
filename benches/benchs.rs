// Quick benches, we should have more of them
// Only works with nightly (benchmarks are still unstable)
#![feature(test)]
extern crate test;
extern crate flat_tree;
extern crate rand;

use test::Bencher;
use rand::distributions::range::Range;
use rand::distributions::IndependentSample;
use rand::{Rng,StdRng,SeedableRng};

use flat_tree::HasChildren;
use flat_tree::FlatTree;
use flat_tree::buffer::FlatTreeIter;

type NodeInner = u64;

#[derive(Debug)]
struct NonFlatNode {
    number: NodeInner,
    children: Vec<NonFlatNode>,
}

impl HasChildren for NonFlatNode {
    fn get_children(&self) -> &[NonFlatNode] {
        &self.children
    }
}

fn generate_tree() -> (NonFlatNode, u64, usize) {
    // A tweak to influence the size of the tree
    // Careful, the tree can grow VERY quickly
    let tweak = 12;

    // Use a fixed seed to have a deterministic tree size
    let mut rng = StdRng::from_seed(&[424242usize]);
    let range = Range::new(4, tweak);

    let mut expected_result = 0;
    let mut n_node = 0;
    let tree = generate_tree_helper(&mut rng, &range, 0, &mut expected_result, &mut n_node);
    (tree, expected_result, n_node)
}

fn generate_tree_helper<T: Rng>(rng: &mut T, range: &Range<u8>, current_depth: u8, expected_result: &mut u64, n_node: &mut usize) -> NonFlatNode {
    let n_children = range.ind_sample(rng).saturating_sub(current_depth);
    let mut children = Vec::with_capacity(n_children as usize);
    for _ in 0..n_children {
        children.push(generate_tree_helper(rng, range, current_depth + 1, expected_result, n_node));
    }
    let number: NodeInner = rng.gen();
    *expected_result += number as u64;
    *n_node += 1;
    NonFlatNode {
        number: number,
        children: children,
    }
}

#[bench]
fn reference(b: &mut Bencher) {
    let (tree, expected_result, _) = generate_tree();
    b.iter(|| {
        let tree = test::black_box(&tree);
        let res = non_flat_recurs(&tree);
        assert_eq!(res, expected_result);
    });
}

fn non_flat_recurs(node: &NonFlatNode) -> u64 {
    let mut res = node.number as u64;
    for child in node.children.iter() {
        res += non_flat_recurs(child);
    }
    res
}

#[bench]
fn flat_bench(b: &mut Bencher) {
    let (tree, expected_result, n) = generate_tree();
    let flat = FlatTree::new(&tree, n, |node| Some(node.number as NodeInner));
    b.iter(|| {
        let flat = test::black_box(&flat);
        let res = flat_recurs(flat.tree_iter());
        assert_eq!(res, expected_result);
    });
}

fn flat_recurs(iter: FlatTreeIter<NodeInner>) -> u64 {
    let mut res = 0;
    for (node, children) in iter {
        res += **node as u64;
        // Prevents too much recursion when we don't need it
        if !children.is_empty() {
            res += flat_recurs(children.children());
        }
    }
    res
}

#[bench]
fn enumerate(b: &mut Bencher) {
    let (tree, expected_result, n) = generate_tree();
    let flat = FlatTree::new(&tree, n, |node| Some(node.number as usize));
    b.iter(|| {
        let mut res = 0u64;
        for node in flat.iter() {
            res += **node as u64;
        }
        assert_eq!(res, expected_result);
    });
}
