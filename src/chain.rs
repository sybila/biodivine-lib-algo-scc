use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
};
use num_bigint::BigInt;

/// does the decomposition of the graph to sccs
/// should be made iterative somethime in the future
/// should also be remade not to return `Vec`
/// also the "metadata" about the graph would be needlessly duplicated this way
/// also colored version wanted (as another method)
pub fn chain(graph: &SymbolicAsyncGraph) -> impl Iterator<Item = GraphColoredVertices> {
    // todo possible to do better than `Vec`?? anyway, iterator does give us the flexibility to change later
    let mut sccs_dump = Vec::new();
    chain_rec(
        graph,
        graph.unit_colored_vertices(),
        graph.empty_colored_vertices(),
        &mut sccs_dump,
    );

    sccs_dump.into_iter()
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
    // todo possible to generate the subgraphs efficiently? better than repeatedly intersecting with the `induced_subgraph_veritces`?
    graph: &SymbolicAsyncGraph,
    induced_subgraph_veritces: &GraphColoredVertices,
    vertices_hint: &GraphColoredVertices,
    sccs_dump: &mut Vec<GraphColoredVertices>,
) {
    if induced_subgraph_veritces.is_empty() {
        return; // base case
    }

    assert!(
        graph.unit_colors().exact_cardinality() == BigInt::from(1), // todo probably use "safer" way than `exact_cardinality()` which may be slow
        "precondition violated; maybe use the colored version instead?" // todo maybe move this into the first recursive call only
    );

    let pivot = match vertices_hint.is_empty() {
        false => vertices_hint,
        true => induced_subgraph_veritces,
    }
    .pick_singleton();

    assert!(!pivot.is_empty()); // trivially true; subgraph is nonempty (else returned above)

    let mut fwd_reachable_acc = graph.mk_empty_colored_vertices();
    let mut current_layer = pivot.clone();
    loop {
        let next_layer = graph
            .post(&current_layer)
            .intersect(induced_subgraph_veritces) // stay in the subgraph
            .minus(&fwd_reachable_acc); // take only the *proper* layer

        if next_layer.is_empty() {
            break;
        }

        fwd_reachable_acc = fwd_reachable_acc.union(&next_layer);
        current_layer = next_layer;
    }

    let fwd_reachable = fwd_reachable_acc;
    let last_fwd_layer = current_layer;

    let mut restricted_bwd_reachable_acc = pivot.clone();
    loop {
        // todo there may be more efficient way to do this wrt. bdd operation efficiency (`pre` on just the last layer?)
        //  cannot use the already available `graph.reach_forward()`;
        //  must intersect with `fwd_reachable` on each step;
        //  unless efficient way of computing induced subgraph
        let resticted_pre = graph // not really a proper *layer*; not cleaned (`.minus(...)`)
            .pre(&restricted_bwd_reachable_acc)
            .intersect(&fwd_reachable);

        if resticted_pre.is_subset(&restricted_bwd_reachable_acc) {
            break; // no further progress possible
        }

        restricted_bwd_reachable_acc = restricted_bwd_reachable_acc.union(&resticted_pre);
    }

    let the_scc = restricted_bwd_reachable_acc;

    // output the scc
    sccs_dump.push(the_scc.clone()); // todo reorder -> no clone (currently readability++)

    let fwd_subgraph = fwd_reachable.minus(&the_scc);
    let fwd_hint = last_fwd_layer.minus(&the_scc);
    chain_rec(graph, &fwd_subgraph, &fwd_hint, sccs_dump);

    let rest_subgraph = induced_subgraph_veritces.minus(&fwd_reachable);
    let scc_predecessors = graph
        .variables()
        .fold(graph.mk_empty_colored_vertices(), |acc, var_id| {
            acc.union(&graph.var_pre(var_id, &the_scc))
        })
        .intersect(induced_subgraph_veritces); // todo intersection necessary?
    let rest_hint = scc_predecessors.minus(&fwd_reachable); // todo `.minus(&the_scc)` correct? more efficient?
    chain_rec(graph, &rest_subgraph, &rest_hint, sccs_dump);
}

#[cfg(test)]
mod test {
    use biodivine_lib_param_bn::{
        biodivine_std::traits::Set, symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork,
        FnUpdate, RegulatoryGraph,
    };
    use num_bigint::{BigInt, Sign};

    use crate::chain::chain;

    #[test]
    fn chain_rec_test() {
        let regulatory_graph = RegulatoryGraph::try_from(
            r#"
            A -| A
            B -> B
            "#,
        )
        .unwrap();

        let var_a = regulatory_graph.find_variable("A").unwrap();
        let var_b = regulatory_graph.find_variable("B").unwrap();

        let mut bool_network = BooleanNetwork::new(regulatory_graph);

        bool_network
            .set_update_function(var_a, Some(FnUpdate::Not(Box::new(FnUpdate::Var(var_a)))))
            .unwrap();

        bool_network
            .set_update_function(var_b, Some(FnUpdate::Var(var_b)))
            .unwrap();

        let async_graph = SymbolicAsyncGraph::new(&bool_network).unwrap();

        let unit_set = async_graph.unit_colored_vertices();

        let a_true = unit_set.fix_network_variable(var_a, true);
        let b_true = unit_set.fix_network_variable(var_b, true);
        let a_false = unit_set.fix_network_variable(var_a, false);
        let b_false = unit_set.fix_network_variable(var_b, false);

        assert!(a_true.exact_cardinality() == BigInt::new(Sign::Plus, vec![2]));
        assert!(b_true.exact_cardinality() == BigInt::new(Sign::Plus, vec![2]));
        assert!(a_false.exact_cardinality() == BigInt::new(Sign::Plus, vec![2]));
        assert!(b_false.exact_cardinality() == BigInt::new(Sign::Plus, vec![2]));

        let false_false = a_false.intersect(&b_false);
        let false_true = a_false.intersect(&b_true);
        let true_false = a_true.intersect(&b_false);
        let true_true = a_true.intersect(&b_true);

        false_false.as_bdd().cardinality();

        assert_eq!(
            false_false.exact_cardinality(),
            BigInt::new(Sign::Plus, vec![1])
        );
        assert_eq!(
            false_true.exact_cardinality(),
            BigInt::new(Sign::Plus, vec![1])
        );
        assert_eq!(
            true_false.exact_cardinality(),
            BigInt::new(Sign::Plus, vec![1])
        );
        assert_eq!(
            true_true.exact_cardinality(),
            BigInt::new(Sign::Plus, vec![1])
        );

        let false_false_post = async_graph.post(&false_false);
        assert_eq!(
            false_false_post.exact_cardinality(),
            BigInt::new(Sign::Plus, vec![1])
        );
        assert_eq!(false_false_post, true_false);

        let a_false_post = async_graph.post(&a_false);
        assert_eq!(
            a_false_post.exact_cardinality(),
            BigInt::new(Sign::Plus, vec![2])
        );
        assert_eq!(a_false_post, a_true);

        // the chain part
        println!(
            "the colors: {:?}",
            async_graph.unit_colors().exact_cardinality()
        );

        let sccs = chain(&async_graph).collect::<Vec<_>>();

        assert_eq!(sccs.len(), 2);

        // one of the components is { (a=false, b=false), (a=true, b=false) }
        assert!(sccs.contains(&b_false));
        // the other is { (a=false, b=true), (a=true, b=true) }
        assert!(sccs.contains(&b_true));
    }
}
