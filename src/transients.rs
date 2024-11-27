use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphVertices, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::{ExtendedBoolean, Space};

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

/// Returns `true` if the `set` is *long-lived*, the opposite of *universally transient*.
/// See also [is_universally_transient].
pub fn is_long_lived(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> bool {
    !is_universally_transient(graph, set)
}

/// Returns `true` if the `set` is *trapped*, i.e. it cannot be escaped by a transition
/// within the given `graph`.
pub fn is_trapped(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> bool {
    for var in graph.variables() {
        let can_go_out = graph.var_can_post_out(var, set);
        if !can_go_out.is_empty() {
            return false;
        }
    }
    true
}

/// Returns the smallest enclosing subspace, as long as the set is not empty.
pub fn enclosing_subspace(graph: &SymbolicAsyncGraph, set: &GraphVertices) -> Space {
    let ctx = graph.symbolic_context();
    let mut space = Space::new_raw(ctx.num_state_variables());
    for var in ctx.network_variables() {
        let bdd_var = ctx.get_state_variable(var);
        let true_subset = set.as_bdd().var_select(bdd_var, true);
        let false_subset = set.as_bdd().var_select(bdd_var, false);
        assert!(!true_subset.is_false() || !false_subset.is_false());
        match (true_subset.is_false(), false_subset.is_false()) {
            (true, true) => unreachable!("The set is empty!"),
            (false, false) => space[var] = ExtendedBoolean::Any,
            (false, true) => space[var] = ExtendedBoolean::One,
            (true, false) => space[var] = ExtendedBoolean::Zero,
        }
    }
    space
}

/// Returns true if `a` is a subspace of `b`, meaning every value that is fixed in `b` is fixed
/// to the same value in `a`.
pub fn is_subspace(a: &Space, b: &Space) -> bool {
    for (var, value) in b.to_values() {
        if a[var] != ExtendedBoolean::from(value) {
            return false;
        }
    }
    true
}
