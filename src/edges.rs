use std::collections::HashMap;

use crate::program::Program;
use crate::syntax_tree::ASTNode;

pub type Edges = HashMap<usize, Vec<usize>>;

pub fn show_edges(parent: &ASTNode, edges: &Edges) {
    edges.iter().for_each(|(key, value)| {
        let node = parent.get_node_by_id(*key);
        if node.is_some() {
            println!("{:#?} -> {:#?}", node.unwrap().code,
                     value
                         .iter()
                         .map(|x| parent.get_node_by_id(*x).
                             unwrap_or(&ASTNode::default()).code.clone())
                         .collect::<Vec<_>>());
        }
    });
}

pub fn show_edges_multiple_programs(parents: &Vec<&Program>, edges: &Edges) {
    edges.iter().for_each(|(key, value)| {
        for parent in parents {
            let node = &parent.tree.get_node_by_id(*key);
            if node.is_some() {
                println!("{:#?} -> {:#?}", node.unwrap().code,
                         value
                             .iter()
                             .map(|x| Program::get_node_by_id_multiple_programs(parents, *x).
                                 unwrap_or(&ASTNode::default()).code.clone())
                             .collect::<Vec<_>>());
            }
        }
    });
}

pub trait Merge {
    fn merge(&mut self, other: &Edges);
}

impl Merge for Edges {
    fn merge(self: &mut Edges, new_edges: &Edges) {
        new_edges.iter().for_each(|(key, value)| {
            if self.contains_key(key) {
                self.get_mut(key).unwrap().extend(value.iter().cloned());
            } else {
                self.insert(*key, value.clone());
            }
        });
    }
}