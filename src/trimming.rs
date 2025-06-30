//! The point of trimming is to remove all trivial SCCs that we can easily detect.
//!
//! This means that the vertex has either no predecessors or no successors within the
//! candidate set (as such, it cannot be a member of any cycle).
//!
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

/// Component of trimming the *easy to detect* trivial SCCs.
///
/// Filters out those states from the `set` that do not have a successor
/// (therefore cannot be a member of a cycle within `set`).
///
/// Does this filtering repeatedly, so even *paths* of nodes that do not have a
/// transitive successor in a cycle will get eliminated.
///
/// Returns the rest of the nodes from the `set` - an overapproximation of the
/// nodes that are in *non-trivial* components.
///
/// (A trivial SCC may still be between two non-trivial SCCs - such will not be
/// detected, hence *overapproximation*)
fn trim_trailing(graph: &SymbolicAsyncGraph, set: GraphColoredVertices) -> GraphColoredVertices {
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

/// Component of trimming the *easy to detect* trivial SCCs.
///
/// Filters out those states from the `set` that do not have a predecessor
/// (therefore cannot be a member of a cycle within the `set`).
///
/// Returns the rest of the nodes from the `set` - an overapproximation of the
/// nodes that are in *non-trivial* components.
///
/// (A trivial SCC may still be between two non-trivial SCCs - such will not be
/// detected, hence *overapproximation*)
fn trim_leading(graph: &SymbolicAsyncGraph, set: GraphColoredVertices) -> GraphColoredVertices {
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

/// Trims *easy to detect* *trivial SCCs*.
///
/// Filters out those states from the `set` that are not on a cycle (therefore
/// cannot be a member of a *non-trivial SCC*).
///
/// Returns the rest of the nodes from the `set` - an overapproximation of the
/// nodes that are in *non-trivial* components.
///
/// (A trivial SCC may still be between two non-trivial SCCs - such will not be
/// detected, hence *overapproximation*)
pub(crate) fn trim(graph: &SymbolicAsyncGraph, set: GraphColoredVertices) -> GraphColoredVertices {
    trim_trailing(graph, trim_leading(graph, set))
}
