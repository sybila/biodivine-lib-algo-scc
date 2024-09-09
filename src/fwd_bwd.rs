//! used just for integration tests - to compare the output of chain on large (non-manual) datasets

use crate::precondition_graph_not_colored;
use crate::reachability::{bwd_saturation, fwd_saturation, naive_bwd, naive_fwd};
use crate::trimming::{trim_bwd_naive, trim_fwd_naive};
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
};
use log::debug;

pub fn fwd_bwd_scc_decomposition_naive(
    graph: &SymbolicAsyncGraph,
) -> impl Iterator<Item = GraphColoredVertices> {
    precondition_graph_not_colored(graph);

    let mut scc_dump = Vec::new();
    let mut remaining_space = graph.mk_unit_colored_vertices();

    debug!(target: "fwd-bwd", "Start SCC decomposition with {} state(s).", remaining_space.exact_cardinality());

    while !remaining_space.is_empty() {
        let scc = get_some_scc_naive(graph, &remaining_space);

        remaining_space = remaining_space.minus(&scc);

        debug!(
            target: "fwd-bwd",
            "Found SCC with {} state(s). Remaining: {}.",
            scc.exact_cardinality(),
            remaining_space.exact_cardinality()
        );

        if !scc.is_singleton() {
            scc_dump.push(scc);
        }
    }

    debug!(
        target: "fwd-bwd",
        "Finished with {} SCCs.",
        scc_dump.len()
    );

    scc_dump.into_iter()
}

pub fn fwd_bwd_scc_decomposition_naive_trim(
    graph: &SymbolicAsyncGraph,
) -> impl Iterator<Item = GraphColoredVertices> {
    precondition_graph_not_colored(graph);

    let mut scc_dump = Vec::new();
    let mut remaining_space = graph.mk_unit_colored_vertices();
    debug!(target: "fwd-bwd-trim", "Start SCC decomposition with {} state(s).", remaining_space.exact_cardinality());

    remaining_space = trim_bwd_naive(graph, &remaining_space);
    remaining_space = trim_fwd_naive(graph, &remaining_space);

    while !remaining_space.is_empty() {
        let scc = get_some_scc_naive(graph, &remaining_space);

        remaining_space = remaining_space.minus(&scc);
        remaining_space = trim_bwd_naive(graph, &remaining_space);
        remaining_space = trim_fwd_naive(graph, &remaining_space);

        debug!(
            target: "fwd-bwd-trim",
            "Found SCC with {} state(s). Remaining: {}.",
            scc.exact_cardinality(),
            remaining_space.exact_cardinality()
        );

        if !scc.is_singleton() {
            scc_dump.push(scc);
        }
    }

    debug!(
        target: "fwd-bwd-trim",
        "Finished with {} SCCs.",
        scc_dump.len()
    );

    scc_dump.into_iter()
}

fn get_some_scc_naive(
    graph: &SymbolicAsyncGraph,
    space_to_pick_from: &GraphColoredVertices,
) -> GraphColoredVertices {
    assert!(!space_to_pick_from.is_empty());

    let pivot = space_to_pick_from.pick_singleton();

    let fwd = naive_fwd(graph, &pivot);
    let bwd = naive_bwd(graph, &pivot);

    debug!(
        target: "get-scc",
        "Forward set has {} and backward set has {} state(s).",
        fwd.exact_cardinality(),
        bwd.exact_cardinality(),
    );

    fwd.intersect(&bwd)
}

// This seems to be a lot of code duplication, but for now I am ok with it in case the
// two algorithms diverge in the future.
pub fn fwd_bwd_scc_decomposition_saturation(
    graph: &SymbolicAsyncGraph,
) -> impl Iterator<Item = GraphColoredVertices> {
    precondition_graph_not_colored(graph);

    let mut scc_dump = Vec::new();
    let mut remaining_space = graph.mk_unit_colored_vertices();

    debug!(target: "fwd-bwd-saturation", "Start SCC decomposition with {} state(s).", remaining_space.exact_cardinality());

    while !remaining_space.is_empty() {
        let scc = get_some_scc_saturation(graph, &remaining_space);

        remaining_space = remaining_space.minus(&scc);

        debug!(
            target: "fwd-bwd-saturation",
            "Found SCC with {} state(s). Remaining: {}.",
            scc.exact_cardinality(),
            remaining_space.exact_cardinality()
        );

        if !scc.is_singleton() {
            scc_dump.push(scc);
        }
    }

    debug!(
        target: "fwd-bwd-saturation",
        "Finished with {} SCCs.",
        scc_dump.len()
    );

    scc_dump.into_iter()
}

pub fn fwd_bwd_scc_decomposition_saturation_trim(
    graph: &SymbolicAsyncGraph,
) -> impl Iterator<Item = GraphColoredVertices> {
    precondition_graph_not_colored(graph);

    let mut scc_dump = Vec::new();
    let mut remaining_space = graph.mk_unit_colored_vertices();
    debug!(target: "fwd-bwd-saturation-trim", "Start SCC decomposition with {} state(s).", remaining_space.exact_cardinality());

    remaining_space = trim_bwd_naive(graph, &remaining_space);
    remaining_space = trim_fwd_naive(graph, &remaining_space);

    while !remaining_space.is_empty() {
        let scc = get_some_scc_saturation(graph, &remaining_space);

        remaining_space = remaining_space.minus(&scc);
        remaining_space = trim_bwd_naive(graph, &remaining_space);
        remaining_space = trim_fwd_naive(graph, &remaining_space);

        debug!(
            target: "fwd-bwd-saturation-trim",
            "Found SCC with {} state(s). Remaining: {}.",
            scc.exact_cardinality(),
            remaining_space.exact_cardinality()
        );

        if !scc.is_singleton() {
            scc_dump.push(scc);
        }
    }

    debug!(
        target: "fwd-bwd-saturation-trim",
        "Finished with {} SCCs.",
        scc_dump.len()
    );

    scc_dump.into_iter()
}

fn get_some_scc_saturation(
    graph: &SymbolicAsyncGraph,
    space_to_pick_from: &GraphColoredVertices,
) -> GraphColoredVertices {
    assert!(!space_to_pick_from.is_empty());

    let pivot = space_to_pick_from.pick_singleton();

    let fwd = fwd_saturation(graph, &pivot);
    let bwd = bwd_saturation(graph, &pivot);

    debug!(
        target: "get-scc-saturation",
        "Forward set has {} and backward set has {} state(s).",
        fwd.exact_cardinality(),
        bwd.exact_cardinality(),
    );

    fwd.intersect(&bwd)
}
