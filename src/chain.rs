use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, GraphVertices, SymbolicAsyncGraph},
};

fn chain() {
    todo!();
}

struct ChainCalculator;

impl ChainCalculator {
    /// does the decomposition of the graph to sccs
    /// should be made iterative somethime in the future
    /// should also be remade not to return `Vec`
    /// also the "metadata" about the graph would be needlessly duplicated this way
    /// also colored version wanted (as another method)
    pub fn chain(_graph: &SymbolicAsyncGraph) -> Vec<SymbolicAsyncGraph> {
        todo!();
    }

    /// recursive version of the chain decomposition
    /// expects all the args to be of the same graph (given by the first parameter)
    ///
    /// only works on a graph with a single color (aka no colors)
    ///
    /// # Arguments
    ///
    /// * `graph` - the graph to be decomposed
    /// * `vertices_hint` - the vertices that are already in the scc
    /// * `sccs_dump` - used to "output" the sccs
    fn chain_rec(
        // todo how to generate the subgraphs efficiently?
        _graph: &SymbolicAsyncGraph,
        // todo remember to always intersect with this one after performing pred/post -> do not escape this subgraph
        _induced_subgraph_veritces: &GraphColoredVertices,
        _vertices_hint: &GraphColoredVertices,
        _sccs_dump: &mut Vec<GraphColoredVertices>,
    ) {
        assert!(
            _graph.unit_colors().symbolic_size() == 1,
            "precondition violated; maybe use the colored version instead?" // todo maybe move this into the first recursive call only
        );

        let pivot = match _vertices_hint.is_empty() {
            false => _vertices_hint,
            true => _induced_subgraph_veritces,
        }
        .pick_singleton();

        assert!(!pivot.is_empty()); // trivially true; subgraph is nonempty (else returned above)

        let mut fwd_reachable_acc = _graph.mk_empty_colored_vertices();
        let mut current_layer = pivot.clone();
        loop {
            let next_layer = _graph
                // todo better way of computing succs over all variables?
                .variables()
                .fold(_graph.mk_empty_colored_vertices(), |acc, var_id| {
                    acc.union(&_graph.var_post(var_id, &current_layer))
                })
                // might want other way of inducing the subgraph, so that there are no such invalid edges
                // for now, defensively intersect // todo do not forget this has to be everywhere
                .intersect(_induced_subgraph_veritces);

            if next_layer.is_empty() {
                break;
            }

            fwd_reachable_acc = fwd_reachable_acc.union(&next_layer);
            current_layer = next_layer;
        }

        let fwd_reachable = fwd_reachable_acc;
        let last_fwd_layer = current_layer;

        let mut restricted_bwd_reachable_acc = _graph.mk_empty_colored_vertices();
        loop {
            let resticted_previous_layer = _graph
                .variables()
                .clone()
                .fold(_graph.mk_empty_colored_vertices(), |acc, var_id| {
                    acc.union(&_graph.var_pre(var_id, &last_fwd_layer)) // todo why `last_fwd_layer`??
                })
                // restrict to just the vertices that are in the scc
                .intersect(&fwd_reachable);

            if resticted_previous_layer.is_subset(&restricted_bwd_reachable_acc) {
                break; // no further progress possible
            }

            restricted_bwd_reachable_acc =
                restricted_bwd_reachable_acc.union(&resticted_previous_layer);
        }

        let the_scc = restricted_bwd_reachable_acc;

        // output the scc
        _sccs_dump.push(the_scc.clone()); // todo reorder -> no clone (currently readability++)

        let fwd_subgraph = fwd_reachable.minus(&the_scc);
        let fwd_hint = last_fwd_layer.minus(&the_scc);
        Self::chain_rec(_graph, &fwd_subgraph, &fwd_hint, _sccs_dump);

        let rest_subgraph = _induced_subgraph_veritces.minus(&fwd_reachable);
        let scc_predecessors = _graph
            .variables()
            .fold(_graph.mk_empty_colored_vertices(), |acc, var_id| {
                acc.union(&_graph.var_pre(var_id, &the_scc))
            })
            .intersect(_induced_subgraph_veritces); // todo intersection necessary?
        let rest_hint = scc_predecessors.minus(&fwd_reachable); // todo `.minus(&the_scc)` correct? more efficient?
        Self::chain_rec(_graph, &rest_subgraph, &rest_hint, _sccs_dump);
    }
}
