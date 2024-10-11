use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Returns `true` if there exists a variable `v` such that all vertices in the `set` can leave
/// the `set` by updating variable `v`.
///
/// Note that when `set` is universally transient, this does not always mean that `v` has the same
/// value across all vertices in `set`. However, any strongly connected subset `X` of `set` is
/// then also universally transient and `v` is constant in such `X`. This is because any strongly
/// connected subset where `v` is not constant needs to at least once update `v` in such a
/// way that the transition does not leave `X` (and hence does not leave `set`). So if `set`
/// can be *always* escaped by updating `v`, no SCC within `set` can have `v` as
/// an oscillating variable.    
pub fn is_universally_transient(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> bool {
    for var in graph.variables() {
        let can_go_out = graph.var_can_post_out(var, set);
        if set.is_subset(&can_go_out) {
            return true;
        }
    }
    false
}
