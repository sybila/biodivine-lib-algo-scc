//! used just for integration tests - to compare the output of chain on large (non-manual) datasets

use crate::precondition_graph_not_colored;
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
};

pub fn fwd_bwd_scc_decomposition(
    graph: &SymbolicAsyncGraph,
) -> impl Iterator<Item = GraphColoredVertices> {
    precondition_graph_not_colored(graph);

    let mut scc_dump = Vec::new();
    let mut remaining_space = graph.mk_unit_colored_vertices();
    while !remaining_space.is_empty() {
        let scc = get_some_scc(graph, &remaining_space);
        remaining_space = remaining_space.minus(&scc);
        scc_dump.push(scc);
    }

    scc_dump.into_iter()
}

fn get_some_scc(
    graph: &SymbolicAsyncGraph,
    space_to_pick_from: &GraphColoredVertices,
) -> GraphColoredVertices {
    assert!(!space_to_pick_from.is_empty());

    let pivot = space_to_pick_from.pick_singleton();

    // let fwd = graph.reach_forward(&pivot);
    // let bwd = graph.reach_backward(&pivot);

    let fwd = naive_fwd(graph, &pivot);
    let bwd = naive_bwd(graph, &pivot);

    fwd.intersect(&bwd)
}

// SymbolicAsyncGraph::reach_forward optimized; use this naive approach for better comparison
fn naive_fwd(graph: &SymbolicAsyncGraph, pivot: &GraphColoredVertices) -> GraphColoredVertices {
    let mut result = pivot.clone();
    let mut curr_layer = pivot.clone();

    loop {
        let next_layer = graph.post(&curr_layer).minus(&result);
        if next_layer.is_empty() {
            break;
        }

        result = result.union(&next_layer);
        curr_layer = next_layer;
    }

    result
}

// SymbolicAsyncGraph::reach_backward optimized; use this naive approach for better comparison
fn naive_bwd(graph: &SymbolicAsyncGraph, pivot: &GraphColoredVertices) -> GraphColoredVertices {
    let mut result = pivot.clone();
    let mut curr_layer = pivot.clone();

    loop {
        let next_layer = graph.pre(&curr_layer).minus(&result);
        if next_layer.is_empty() {
            break;
        }

        result = result.union(&next_layer);
        curr_layer = next_layer;
    }

    result
}
