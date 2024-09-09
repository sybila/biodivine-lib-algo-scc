//! The point of trimming is to remove all trivial SCCs that we can easily detect.
//!
//! This means that the vertex has either no predecessors or no successors within the
//! candidate set (as such, it cannot be a member of any cycle).
//!
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use log::{debug, trace};

pub fn trim_fwd_naive(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
) -> GraphColoredVertices {
    debug!(
        target: "trim-fwd-naive",
        "Start naive forward trimming of {} state(s).",
        set.exact_cardinality()
    );

    let mut result = set.clone();

    loop {
        let has_successor = graph.can_post_within(&result);

        if result.is_subset(&has_successor) {
            debug!(
                target: "trim-fwd-naive",
                "Trimming ended with {} state(s).",
                set.exact_cardinality()
            );

            return result;
        }

        result = has_successor;

        trace!(
            target: "trim-fwd-naive",
            "Result trimmed to {}[bdd_nodes:{}]",
            result.exact_cardinality(),
            result.symbolic_size()
        );
    }
}

pub fn trim_bwd_naive(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
) -> GraphColoredVertices {
    debug!(
        target: "trim-bwd-naive",
        "Start naive backward trimming of {} state(s).",
        set.exact_cardinality()
    );

    let mut result = set.clone();

    loop {
        let has_predecessor = graph.can_pre_within(&result);

        if result.is_subset(&has_predecessor) {
            debug!(
                target: "trim-bwd-naive",
                "Trimming ended with {} state(s).",
                set.exact_cardinality()
            );

            return result;
        }

        result = has_predecessor;

        trace!(
            target: "trim-bwd-naive",
            "Result trimmed to {}[bdd_nodes:{}]",
            result.exact_cardinality(),
            result.symbolic_size()
        );
    }
}
