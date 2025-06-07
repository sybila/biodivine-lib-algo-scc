use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use log::{debug, trace};

// SymbolicAsyncGraph::reach_forward optimized; use this naive approach for better comparison
pub fn naive_fwd(graph: &SymbolicAsyncGraph, pivot: &GraphColoredVertices) -> GraphColoredVertices {
    debug!(
        target: "reach-fwd-naive",
        "Starting (naive) forward reachability of {} state(s).",
        pivot.exact_cardinality()
    );
    let mut result = pivot.clone();
    let mut curr_layer = pivot.clone();

    loop {
        let next_layer = graph.post(&curr_layer).minus(&result);
        if next_layer.is_empty() {
            break;
        }

        result = result.union(&next_layer);
        curr_layer = next_layer;

        trace!(
            target: "reach-fwd-naive",
            "Result increased to {}[bdd_nodes:{}].",
            result.exact_cardinality(),
            result.symbolic_size()
        );
    }

    debug!(
        target: "reach-bwd-naive",
        "Forward reachability finished with {} state(s).",
        result.exact_cardinality()
    );

    result
}

// SymbolicAsyncGraph::reach_backward optimized; use this naive approach for better comparison
pub fn naive_bwd(graph: &SymbolicAsyncGraph, pivot: &GraphColoredVertices) -> GraphColoredVertices {
    debug!(
        target: "reach-bwd-naive",
        "Starting (naive) backward reachability of {} state(s).",
        pivot.exact_cardinality()
    );

    let mut result = pivot.clone();
    let mut curr_layer = pivot.clone();

    loop {
        let next_layer = graph.pre(&curr_layer).minus(&result);
        if next_layer.is_empty() {
            break;
        }

        result = result.union(&next_layer);
        curr_layer = next_layer;

        trace!(
            target: "reach-bwd-naive",
            "Result increased to {}[bdd_nodes:{}].",
            result.exact_cardinality(),
            result.symbolic_size()
        );
    }

    debug!(
        target: "reach-bwd-naive",
        "Backward reachability finished with {} state(s).",
        result.exact_cardinality()
    );

    result
}

pub fn bwd_saturation(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
) -> GraphColoredVertices {
    debug!(target: "reach-bwd-saturation", "Starting backward saturation of {} state(s).", initial.exact_cardinality());

    let mut result_accumulator = initial.clone();

    let rev_variables = graph.variables().rev().collect::<Vec<_>>();

    'from_bottom_var: loop {
        for var in rev_variables.iter() {
            let step = graph.var_pre_out(*var, &result_accumulator);

            if !step.is_empty() {
                result_accumulator = result_accumulator.union(&step);

                trace!(
                    target: "reach-bwd-saturation",
                    "Result increased to {}[bdd_nodes:{}].",
                    result_accumulator.exact_cardinality(),
                    result_accumulator.symbolic_size()
                );

                continue 'from_bottom_var;
            }
        }

        debug!(
            target: "reach-bwd-saturation",
            "Backward saturation finished with {} state(s).",
            result_accumulator.exact_cardinality()
        );

        break result_accumulator;
    }
}

pub fn fwd_saturation(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
) -> GraphColoredVertices {
    debug!(target: "reach-fwd-saturation", "Starting forward saturation of {} state(s).", initial.exact_cardinality());

    let mut result_accumulator = initial.clone();

    let rev_variables = graph.variables().rev().collect::<Vec<_>>();

    'from_bottom_var: loop {
        for var in rev_variables.iter() {
            let step = graph.var_post_out(*var, &result_accumulator);

            if !step.is_empty() {
                result_accumulator = result_accumulator.union(&step);

                trace!(
                    target: "reach-fwd-saturation",
                    "Result increased to {}[bdd_nodes:{}].",
                    result_accumulator.exact_cardinality(),
                    result_accumulator.symbolic_size()
                );

                continue 'from_bottom_var;
            }
        }

        debug!(
            target: "reach-fwd-saturation",
            "Forward saturation finished with {} state(s).",
            result_accumulator.exact_cardinality()
        );

        break result_accumulator;
    }
}
