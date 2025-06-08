//! The point of trimming is to remove all trivial SCCs that we can easily detect.
//!
//! This means that the vertex has either no predecessors or no successors within the
//! candidate set (as such, it cannot be a member of any cycle).
//!
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

// todo consider exposing just a `trim_naive` fn, since we never really call these separately (check)

/// Component of trimming the *"easy to detect"* trivial SCCs (thus
/// `naive`).
///
/// Returns only those states that have successors, therefore may be a member
/// of a cycle within `set`.
pub(crate) fn trim_fwd_naive(
    graph: &SymbolicAsyncGraph,
    set: GraphColoredVertices,
) -> GraphColoredVertices {
    match set.is_empty() {
        true => set,
        false => {
            let mut result = set;
            loop {
                let has_successor = graph.can_post_within(&result);
                if result.is_subset(&has_successor) {
                    // no change -> found "base", return
                    break result;
                }

                result = has_successor;
            }
        }
    }
}

/// Component of trimming the *"easy to detect"* trivial SCCs (thus
/// `naive`).
///
/// Returns only those states that have predecessors, therefore may be a member
/// of a cycle within `set`.
pub(crate) fn trim_bwd_naive(
    graph: &SymbolicAsyncGraph,
    set: GraphColoredVertices,
) -> GraphColoredVertices {
    match set.is_empty() {
        true => set,
        false => {
            let mut result = set;
            loop {
                let has_predecessor = graph.can_pre_within(&result);
                if result.is_subset(&has_predecessor) {
                    // no change -> found "base", return
                    break result;
                }

                result = has_predecessor;
            }
        }
    }
}
