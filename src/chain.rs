use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
};

use crate::{assert_precondition_graph_not_colored, hamming::Hamming};

/// does the decomposition of the graph to SCCs
/// should be made iterative sometime in the future
/// should also be remade not to return `Vec`
/// also the "metadata" about the graph would be needlessly duplicated this way
/// also colored version wanted (as another method)
///
/// The order of the output sccs is undefined.
pub fn chain(graph: &SymbolicAsyncGraph) -> impl Iterator<Item = GraphColoredVertices> {
    assert_precondition_graph_not_colored(graph);

    let mut scc_dump = Vec::new();
    // `chain_rec` assumes it can pick a pivot -> must not pass in an empty graph
    match graph.unit_vertices().is_empty() {
        true => { /* no sccs in an empty graph */ }
        false => chain_rec(
            graph,
            graph.empty_colored_vertices(), // no hint
            &mut scc_dump,
        ),
    }
    scc_dump.into_iter()
}

/// recursive version of the chain decomposition
/// expects all the args to be of the same graph (given by the first parameter)
///
/// expects a nonempty graph (must be able to pick a pivot)
///
/// only works on a graph with a single color (aka no colors)
///
/// # Arguments
///
/// * `graph` - the graph to be decomposed
/// * `vertices_hint` - the vertices that are already in the scc
/// * `scc_dump` - used to "output" the SCCs
fn chain_rec(
    graph: &SymbolicAsyncGraph,
    vertices_hint: &GraphColoredVertices,
    scc_dump: &mut Vec<GraphColoredVertices>,
) {
    // todo consider using `debug_assert! everywhere` (test performance gain)
    assert!(!graph.unit_vertices().is_empty());
    assert!(vertices_hint.is_subset(graph.unit_colored_vertices()));

    let pivot_set = match vertices_hint.is_empty() {
        true => graph.unit_colored_vertices(),
        false => vertices_hint,
    };
    let pivot = pivot_set.pick_singleton();

    assert!(!pivot.is_empty()); // trivially true; subgraph is nonempty (else returned above)

    let mut fwd_reachable_acc = pivot.clone();
    let mut current_layer = pivot.clone();
    loop {
        let next_layer = graph.post(&current_layer).minus(&fwd_reachable_acc); // take only the *proper* layer

        if next_layer.is_empty() {
            break;
        }

        fwd_reachable_acc = fwd_reachable_acc.union(&next_layer);
        current_layer = next_layer;
    }

    let fwd_reachable = fwd_reachable_acc;
    let last_fwd_layer = current_layer;

    let mut restricted_bwd_reachable_acc = pivot.clone();
    let graph_fwd_restricted = graph.restrict(&fwd_reachable);
    loop {
        let restricted_pre = graph_fwd_restricted // not really a proper *layer*; not cleaned (`.minus(...)`)
            .pre(&restricted_bwd_reachable_acc);

        if restricted_pre.is_subset(&restricted_bwd_reachable_acc) {
            break; // no further progress possible
        }

        restricted_bwd_reachable_acc = restricted_bwd_reachable_acc.union(&restricted_pre);
    }

    let the_scc = restricted_bwd_reachable_acc;

    // Output the scc.
    // todo perf move to the end of this fn to avoid clone (mind the inversed order)
    if !the_scc.is_singleton() {
        // todo this filter should probably be a part of a config/parameter
        scc_dump.push(the_scc.clone());
    }

    let fwd_remaining = fwd_reachable.minus(&the_scc);
    if !fwd_remaining.is_empty() {
        let fwd_subgraph = graph.restrict(&fwd_remaining);
        let fwd_hint = last_fwd_layer.minus(&the_scc);
        chain_rec(&fwd_subgraph, &fwd_hint, scc_dump);
    }

    let rest_remaining = graph.unit_colored_vertices().minus(&fwd_reachable);
    if !rest_remaining.is_empty() {
        let rest_subgraph = graph.restrict(&rest_remaining);
        let rest_hint = graph.pre(&the_scc).intersect(&rest_remaining);
        chain_rec(&rest_subgraph, &rest_hint, scc_dump);
    }
}

/// Does the decomposition of the graph to SCCs,
/// but using the *saturation* strategy to perform the forward and backward
/// reachability.
///
/// The order of the output sccs is undefined.
pub fn chain_saturation(graph: &SymbolicAsyncGraph) -> impl Iterator<Item = GraphColoredVertices> {
    assert_precondition_graph_not_colored(graph);

    let mut scc_dump = Vec::new();
    // `chain_rec` assumes it can pick a pivot -> must not pass in an empty graph
    match graph.unit_vertices().is_empty() {
        true => { /* no sccs in an empty graph */ }
        false => chain_rec_saturation(
            graph,
            graph.empty_colored_vertices(), // no hint
            &mut scc_dump,
        ),
    }
    scc_dump.into_iter()
}

fn fwd_saturation(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut result_accumulator = initial.clone();

    // todo likely better perf, right?
    // todo do not collect
    let rev_variables = graph.variables().rev().collect::<Vec<_>>();

    'from_bottom_var: loop {
        for var in rev_variables.iter() {
            let step = graph.var_post_out(*var, &result_accumulator);

            if !step.is_empty() {
                result_accumulator = result_accumulator.union(&step);

                continue 'from_bottom_var;
            }
        }

        break result_accumulator;
    }
}

fn bwd_saturation(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut result_accumulator = initial.clone();

    // todo likely better perf, right?
    // todo do not collect
    let rev_variables = graph.variables().rev().collect::<Vec<_>>();

    'from_bottom_var: loop {
        for var in rev_variables.iter() {
            let step = graph.var_pre_out(*var, &result_accumulator);

            if !step.is_empty() {
                result_accumulator = result_accumulator.union(&step);

                continue 'from_bottom_var;
            }
        }

        break result_accumulator;
    }
}

fn chain_rec_saturation(
    graph: &SymbolicAsyncGraph,
    vertices_hint: &GraphColoredVertices,
    scc_dump: &mut Vec<GraphColoredVertices>,
) {
    assert!(!graph.unit_vertices().is_empty());

    let pivot_set = match vertices_hint.is_empty() {
        true => graph.unit_colored_vertices(),
        false => vertices_hint,
    };
    let pivot = pivot_set.pick_singleton();

    assert!(!pivot.is_empty()); // trivially true; subgraph is nonempty (else returned above)

    let fwd_reachable = fwd_saturation(graph, &pivot);

    let scc = bwd_saturation(&graph.restrict(&fwd_reachable), &pivot);

    // Output the scc.
    // todo move to the end to avoid clone - beware of the change in the order
    if !scc.is_singleton() {
        // todo this filter should probably be a part of a config/parameter
        scc_dump.push(scc.clone());
    }

    let fwd_remaining = fwd_reachable.minus(&scc);
    if !fwd_remaining.is_empty() {
        let fwd_subgraph = graph.restrict(&fwd_remaining);

        // todo use better heuristic (hamming dist?); want to get close to the "bottom"
        // todo beware of correctness; must pick the hint from somewhere else than the scc
        // let fwd_hint = last_fwd_layer.minus(&the_scc);
        let fwd_hint = fwd_remaining;

        chain_rec_saturation(&fwd_subgraph, &fwd_hint, scc_dump);
    }

    let rest_remaining = graph.unit_colored_vertices().minus(&fwd_reachable);
    if !rest_remaining.is_empty() {
        let rest_subgraph = graph.restrict(&rest_remaining);
        let rest_hint = graph.pre(&scc).intersect(&rest_remaining);
        chain_rec_saturation(&rest_subgraph, &rest_hint, scc_dump);
    }
}

/// Does the decomposition of the graph to SCCs,
/// but using the *saturation* strategy to perform the forward and backward
/// reachability *and* the hamming heuristic for picking the pivot.
///
/// The order of the output sccs is undefined.
pub fn chain_saturation_hamming_heuristic(
    graph: &SymbolicAsyncGraph,
) -> impl Iterator<Item = GraphColoredVertices> {
    assert_precondition_graph_not_colored(graph);

    let mut scc_dump = Vec::new();
    match graph.unit_vertices().is_empty() {
        true => { /* no sccs in an empty graph */ }
        false => chain_rec_saturation_hamming_heuristic(
            graph,
            graph.empty_colored_vertices(), // no hint
            &mut scc_dump,
        ),
    }
    scc_dump.into_iter()
}

// todo consider adding `chain_hamming_heuristic`? - to have the full
// `{with_saturation, without_saturation} x {with_hamming, without_hamming}`
// options?

fn chain_rec_saturation_hamming_heuristic(
    graph: &SymbolicAsyncGraph,
    vertices_hint: &GraphColoredVertices,
    scc_dump: &mut Vec<GraphColoredVertices>,
) {
    assert!(!graph.unit_vertices().is_empty());

    let pivot_set = match vertices_hint.is_empty() {
        true => graph.unit_colored_vertices(),
        false => vertices_hint,
    };
    let pivot = pivot_set.pick_singleton();

    assert!(!pivot.is_empty()); // trivially true; subgraph is nonempty (else returned above)

    let fwd_reachable = fwd_saturation(graph, &pivot);

    let scc = bwd_saturation(&graph.restrict(&fwd_reachable), &pivot);

    // Output the scc.
    // todo
    if !scc.is_singleton() {
        // todo this filter should probably be a part of a config/parameter
        scc_dump.push(scc.clone());
    }

    let fwd_remaining = fwd_reachable.minus(&scc);
    if !fwd_remaining.is_empty() {
        let fwd_subgraph = graph.restrict(&fwd_remaining);

        let fwd_hint = pivot.ham_furthest_within(&fwd_remaining); // <-- the difference

        chain_rec_saturation_hamming_heuristic(&fwd_subgraph, &fwd_hint, scc_dump);
    }

    let rest_remaining = graph.unit_colored_vertices().minus(&fwd_reachable);
    if !rest_remaining.is_empty() {
        let rest_subgraph = graph.restrict(&rest_remaining);
        let rest_hint = graph.pre(&scc).intersect(&rest_remaining);
        chain_rec_saturation_hamming_heuristic(&rest_subgraph, &rest_hint, scc_dump);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use biodivine_lib_param_bn::{
        biodivine_std::traits::Set,
        symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
        BooleanNetwork,
    };
    use num_bigint::BigInt;
    use test_generator::test_resources;

    use crate::{
        chain::{chain, chain_saturation, chain_saturation_hamming_heuristic},
        fwd_bwd::fwd_bwd_scc_decomposition_naive,
    };

    fn basic_async_graph() -> SymbolicAsyncGraph {
        let bool_network = BooleanNetwork::try_from(
            r#"
            A -| A
            B -> B
            $A: !A
            $B: B
            "#,
        )
        .unwrap();
        SymbolicAsyncGraph::new(&bool_network).unwrap()
    }

    fn basic_decomposition<F, I>(decomposition_fn: F)
    where
        F: Fn(&SymbolicAsyncGraph) -> I,
        I: Iterator<Item = GraphColoredVertices>,
    {
        let async_graph = basic_async_graph();
        let mut vars = async_graph.variables();
        let var_a = vars.next().unwrap();
        let var_b = vars.next().unwrap();
        assert!(vars.next().is_none());

        let unit_set = async_graph.unit_colored_vertices();

        let a_true = unit_set.fix_network_variable(var_a, true);
        let b_true = unit_set.fix_network_variable(var_b, true);
        let a_false = unit_set.fix_network_variable(var_a, false);
        let b_false = unit_set.fix_network_variable(var_b, false);

        assert_eq!(a_true.exact_cardinality(), BigInt::from(2));
        assert_eq!(b_true.exact_cardinality(), BigInt::from(2));
        assert_eq!(a_false.exact_cardinality(), BigInt::from(2));
        assert_eq!(b_false.exact_cardinality(), BigInt::from(2));

        let false_false = a_false.intersect(&b_false);
        let false_true = a_false.intersect(&b_true);
        let true_false = a_true.intersect(&b_false);
        let true_true = a_true.intersect(&b_true);

        assert_eq!(false_false.exact_cardinality(), BigInt::from(1));
        assert_eq!(false_true.exact_cardinality(), BigInt::from(1));
        assert_eq!(true_false.exact_cardinality(), BigInt::from(1));
        assert_eq!(true_true.exact_cardinality(), BigInt::from(1));

        let false_false_post = async_graph.post(&false_false);
        assert_eq!(false_false_post.exact_cardinality(), BigInt::from(1));
        assert_eq!(false_false_post, true_false);

        let a_false_post = async_graph.post(&a_false);
        assert_eq!(a_false_post.exact_cardinality(), BigInt::from(2));
        assert_eq!(a_false_post, a_true);

        // the chain part
        println!(
            "the colors: {:?}",
            async_graph.unit_colors().exact_cardinality()
        );

        let scc_vec = decomposition_fn(&async_graph).collect::<Vec<_>>();

        assert_eq!(scc_vec.len(), 2);

        // one of the components is { (a=false, b=false), (a=true, b=false) }
        assert!(scc_vec.contains(&b_false));
        // the other is { (a=false, b=true), (a=true, b=true) }
        assert!(scc_vec.contains(&b_true));
    }

    #[test]
    fn chain_test() {
        basic_decomposition(chain);
    }

    #[test]
    fn chain_saturation_test() {
        basic_decomposition(chain_saturation);
    }

    #[test]
    fn chain_saturation_hamming_heuristic_test() {
        basic_decomposition(chain_saturation_hamming_heuristic);
    }

    #[test]
    fn compare_chain_fwd_bwd_basic_graph() {
        let async_graph = basic_async_graph();

        let chain_scc_set = chain(&async_graph).collect::<HashSet<_>>();
        let fwd_bwd_scc_set = fwd_bwd_scc_decomposition_naive(&async_graph).collect::<HashSet<_>>();

        assert_eq!(chain_scc_set, fwd_bwd_scc_set);
    }

    fn compare_fn_with_fwd_bwd<F, I>(model_path: &str, decomposition_fn: F)
    where
        F: Fn(&SymbolicAsyncGraph) -> I,
        I: Iterator<Item = GraphColoredVertices>,
    {
        let bn = BooleanNetwork::try_from_file(model_path).unwrap();

        let skip_threshold = if cfg!(feature = "expensive-tests") {
            14
        } else {
            10
        };

        if bn.num_vars() > skip_threshold {
            // The network is too large.
            println!(
                " >> [{} > {}] Skipping {}.",
                bn.num_vars(),
                skip_threshold,
                model_path
            );
            return;
        }

        // Network has no parameters (no colors).
        assert_eq!(bn.num_parameters(), 0);
        assert_eq!(bn.num_implicit_parameters(), 0);

        let graph = SymbolicAsyncGraph::new(&bn).unwrap();

        println!(
            " >> [{} <= {}] Testing {}.",
            bn.num_vars(),
            skip_threshold,
            model_path
        );

        println!(" >> Computing FWD-BWD.");
        let fwd_bwd_scc_set = fwd_bwd_scc_decomposition_naive(&graph).collect::<HashSet<_>>();

        println!(" >> Computing with {}.", std::any::type_name::<F>());
        let chain_scc_set = decomposition_fn(&graph).collect::<HashSet<_>>();

        println!(" >> Found {} SCCs.", fwd_bwd_scc_set.len());

        assert_eq!(chain_scc_set, fwd_bwd_scc_set);
    }

    #[test_resources("./models/bbm-inputs-true/*.aeon")]
    fn compare_chain_fwd_bwd_selected(model_path: &str) {
        compare_fn_with_fwd_bwd(model_path, chain);
    }

    #[test_resources("./models/bbm-inputs-true/*.aeon")]
    fn compare_chain_saturation_fwd_bwd_selected(model_path: &str) {
        compare_fn_with_fwd_bwd(model_path, chain_saturation);
    }

    #[test_resources("./models/bbm-inputs-true/*.aeon")]
    fn compare_chain_saturation_hamming_heuristic_fwd_bwd_selected(model_path: &str) {
        compare_fn_with_fwd_bwd(model_path, chain_saturation_hamming_heuristic);
    }
}
