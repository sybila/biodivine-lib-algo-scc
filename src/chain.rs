use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphVertices, SymbolicAsyncGraph},
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
    /// # Arguments
    ///
    /// * `graph` - the graph to be decomposed
    /// * `vertices_hint` - the vertices that are already in the scc
    /// * `sccs_acc` - the accumulator of the sccs
    fn chain_rec(
        // todo how to generate the subgraphs efficiently?
        _graph: &SymbolicAsyncGraph,
        // todo remember to always intersect with this one after performing pred/post -> do not escape this subgraph
        _induced_subgraph_veritces: &GraphVertices,
        _vertices_hint: &GraphVertices,
        _sccs_acc: &mut Vec<GraphVertices>,
    ) {
        let pivot = match _vertices_hint.is_empty() {
            false => _vertices_hint.pick_singleton(),
            true => _induced_subgraph_veritces
                .minus(_vertices_hint)
                .pick_singleton(),
        };

        assert!(!pivot.is_empty());

        let mut fwd_reachable_acc = _graph.empty_vertices();
        let mut current_layer = pivot.clone();
        let mut scc_acc = pivot.clone();

        loop {
            // todo want `unify_over { foreach async }`
            // currently only for colored vertices?
            // let next_layer = _graph
            //     .post(&current_layer)
            //     .intersect_vertices(_induced_subgraph_veritces);
        }

        let vs = _graph.empty_vertices();
        _sccs_acc.push(vs.clone()); // shut up clippy
        todo!();
    }
}
