use std::collections::HashSet;

use crepe::crepe;

use crate::edges::Edges;

crepe! {
    @input
    pub struct FlowEdge(usize, usize);


    @output
    #[derive(Debug)]
    pub struct Flow(usize, usize);

    Flow(x, y) <- FlowEdge(x, y);
    Flow(x, z) <- FlowEdge(x, y), Flow(y, z);
}

pub fn solve(edges: &Edges) -> HashSet<Flow> {
    let mut flow_edges: Vec<FlowEdge> = vec![];
    let mut runtime = Crepe::new();
    for (x, ys) in edges {
        for y in ys {
            flow_edges.push(FlowEdge(*x, *y));
        }
    }
    runtime.extend(flow_edges);
    let (flows, ) = runtime.run();
    flows
}

pub type Domain = HashSet<Flow>;

pub trait Reachable {
    fn is_reachable(&self, a: usize, b: usize) -> bool;
}

impl Reachable for Domain {
    fn is_reachable(&self, a: usize, b: usize) -> bool {
        if self.contains(&Flow(a, b)) {
            return true;
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edges() {
        let mut runtime = Crepe::new();
        runtime.extend(&[FlowEdge(1, 2), FlowEdge(2, 3), FlowEdge(3, 4), FlowEdge(2, 5)]);
        let (flows, ) = runtime.run();
    }
}
