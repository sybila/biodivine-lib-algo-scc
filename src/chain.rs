use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
};

use crate::{assert_precondition_graph_not_colored, hamming::Hamming, trimming::trim};

#[derive(Clone, Copy, Debug, Default)]
pub enum TrimLvl {
    /// Do not trim *trivial SCCs* at all
    #[default]
    None,
    /// Only trim *trivial SCCs* before the start of the decomposition itself.
    /// Some *trivial SCCs* may still make it into the result - namely if an
    /// SCC is on a path between two *non-trivial SCCs*.
    StartOnly,
    /// Trim *trivial SCCs* before each "iteration" of the decomposition.
    /// This ensures all *trivial SCCs* (even those between two *non-trivial
    /// SCCs* are filtered out).
    Full,
}

/// does the decomposition of the graph to SCCs
/// should be made iterative sometime in the future
/// should also be remade not to return `Vec`
/// also the "metadata" about the graph would be needlessly duplicated this way
/// also colored version wanted (as another method)
///
/// The order of the output sccs is undefined.
pub fn chain(
    graph: SymbolicAsyncGraph,
    trim_lvl: TrimLvl,
) -> impl Iterator<Item = GraphColoredVertices> {
    assert_precondition_graph_not_colored(&graph);

    const fn identity(_: &SymbolicAsyncGraph, it: GraphColoredVertices) -> GraphColoredVertices {
        it
    }

    match trim_lvl {
        TrimLvl::None => match graph.unit_vertices().is_empty() {
            true => Default::default(),
            false => {
                let no_hint = graph.empty_colored_vertices().clone();

                chain_iterative(graph, no_hint, /* noop - no trim */ identity)
            }
        },
        TrimLvl::StartOnly => {
            let trimmed = trim(&graph, graph.unit_colored_vertices().clone());
            let graph = graph.restrict(&trimmed);

            match graph.unit_vertices().is_empty() {
                true => Default::default(),
                false => {
                    let no_hint = graph.empty_colored_vertices().clone();
                    chain_iterative(graph, no_hint, /* noop - no trim */ identity)
                }
            }
        }
        TrimLvl::Full => {
            let trimmed = trim(&graph, graph.unit_colored_vertices().clone());
            let graph = graph.restrict(&trimmed);

            match graph.unit_vertices().is_empty() {
                true => Default::default(),
                false => {
                    let no_hint = graph.empty_colored_vertices().clone();
                    chain_iterative(graph, no_hint, /* trim every iteration */ trim)
                }
            }
        }
    }
    .into_iter()
}

/// the chain decomposition
///
///  expects all the args to be of the same graph (given by the first parameter)
///
/// expects a nonempty graph (must be able to pick a pivot)
///
/// only works on a graph with a single color (aka no colors)
///
/// # Arguments
///
/// * `graph` - the graph to be decomposed - must not be empty
/// * `vertices_hint` - the vertices that are already in the scc
/// * `restrictor` - function that further restricts the sets that are to be
///   "recursively" decomposed into SCCs. Pass in `|_, it| it` to ignore this.
fn chain_iterative(
    graph: SymbolicAsyncGraph,
    vertices_hint: GraphColoredVertices,
    restrictor: fn(&SymbolicAsyncGraph, GraphColoredVertices) -> GraphColoredVertices,
) -> Vec<GraphColoredVertices> {
    let mut output = Vec::<GraphColoredVertices>::new();
    let mut stack = vec![(graph, vertices_hint)];

    while let Some((graph, vertices_hint)) = stack.pop() {
        assert!(!graph.unit_vertices().is_empty());
        assert!(vertices_hint.is_subset(graph.unit_colored_vertices()));

        let pivot_set = match vertices_hint.is_empty() {
            true => graph.unit_colored_vertices(),
            false => &vertices_hint,
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

        let mut restricted_bwd_reachable_acc = pivot;
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

        let fwd_remaining = fwd_reachable.minus(&the_scc);
        let fwd_remaining = restrictor(&graph, fwd_remaining);
        if !fwd_remaining.is_empty() {
            let fwd_subgraph = graph.restrict(&fwd_remaining);

            // must intersect with `fwd_remaining`; it might have changed
            // -> no need to `minus scc`; `fwd_remaining` does not contain scc

            // todo `last_layer` and `fwd_remaining` may have diverged
            //  (`restrictor` fn might have "cut off" the whole
            //  `last_fwd_layer` part, their intersection might now be empty;
            //  the alg still can work with *empty hint*, the perf might suffer
            //  tho)
            let fwd_hint = last_fwd_layer.intersect(&fwd_remaining);

            // "recursive call"
            stack.push((fwd_subgraph, fwd_hint));
        }

        let rest_remaining = graph.unit_colored_vertices().minus(&fwd_reachable);
        let rest_remaining = restrictor(&graph, rest_remaining);
        if !rest_remaining.is_empty() {
            let rest_subgraph = graph.restrict(&rest_remaining);

            // todo same as in the other branch; hint might be empty (even in
            // cases `rest_subgraph` is nonempty), in cases restrictor trimmed
            // too much
            let rest_hint = rest_subgraph.pre(&the_scc);

            // "recursive call"
            stack.push((rest_subgraph, rest_hint));
        }

        // Output the scc.
        if !the_scc.is_singleton() {
            // todo this filter should probably be a part of a config/parameter
            // - or use trimming
            output.push(the_scc);
        }
    }

    output
}

/// Does the decomposition of the graph to SCCs,
/// but using the *saturation* strategy to perform the forward and backward
/// reachability.
///
/// The order of the output sccs is undefined.
pub fn chain_saturation(graph: SymbolicAsyncGraph) -> impl Iterator<Item = GraphColoredVertices> {
    assert_precondition_graph_not_colored(&graph);

    // `chain_rec` assumes it can pick a pivot -> must not pass in an empty graph
    match graph.unit_vertices().is_empty() {
        true => Default::default(),
        false => {
            let empty_colored_vertices = graph.empty_colored_vertices().clone();
            _chain_saturation(
                graph,
                empty_colored_vertices, // no hint
            )
        }
    }
    .into_iter()
}

fn fwd_saturation(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut result_accumulator = initial.clone();

    // better to collect; there is some computation to producing `variables()`
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

    // better to collect; there is some computation to producing `variables()`
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

fn _chain_saturation(
    graph: SymbolicAsyncGraph,
    vertices_hint: GraphColoredVertices,
) -> Vec<GraphColoredVertices> {
    let mut output = Vec::<GraphColoredVertices>::new();
    let mut stack = vec![(graph, vertices_hint)];

    while let Some((graph, vertices_hint)) = stack.pop() {
        assert!(!graph.unit_vertices().is_empty());

        let pivot_set = match vertices_hint.is_empty() {
            true => graph.unit_colored_vertices(),
            false => &vertices_hint,
        };
        let pivot = pivot_set.pick_singleton();

        assert!(!pivot.is_empty()); // trivially true; subgraph is nonempty (else returned above)

        let fwd_reachable = fwd_saturation(&graph, &pivot);

        let scc = bwd_saturation(&graph.restrict(&fwd_reachable), &pivot);

        let fwd_remaining = fwd_reachable.minus(&scc);
        if !fwd_remaining.is_empty() {
            let fwd_subgraph = graph.restrict(&fwd_remaining);
            // no better estimate in this implementation
            let fwd_hint = fwd_remaining;

            // chain_rec_saturation(&fwd_subgraph, &fwd_hint, scc_dump);
            stack.push((fwd_subgraph, fwd_hint));
        }

        let rest_remaining = graph.unit_colored_vertices().minus(&fwd_reachable);
        if !rest_remaining.is_empty() {
            let rest_subgraph = graph.restrict(&rest_remaining);
            let rest_hint = graph.pre(&scc).intersect(&rest_remaining);

            // chain_rec_saturation(&rest_subgraph, &rest_hint, scc_dump);
            stack.push((rest_subgraph, rest_hint));
        }

        // Output the scc.
        if !scc.is_singleton() {
            // todo this filter should probably be a part of a config/parameter
            output.push(scc);
        }
    }

    output
}

/// Does the decomposition of the graph to SCCs,
/// but using the *saturation* strategy to perform the forward and backward
/// reachability *and* the hamming heuristic for picking the pivot.
///
/// The order of the output sccs is undefined.
pub fn chain_saturation_hamming_heuristic(
    graph: SymbolicAsyncGraph,
) -> impl Iterator<Item = GraphColoredVertices> {
    assert_precondition_graph_not_colored(&graph);

    // `chain_rec` assumes it can pick a pivot -> must not pass in an empty graph
    match graph.unit_vertices().is_empty() {
        true => Default::default(),
        false => {
            let empty_colored_vertices = graph.empty_colored_vertices().clone();
            _chain_saturation_hamming_heuristic(
                graph,
                empty_colored_vertices, // no hint
            )
        }
    }
    .into_iter()
}

// todo consider adding `chain_hamming_heuristic`? - to have the full
// `{with_saturation, without_saturation} x {with_hamming, without_hamming}`
// options?

fn _chain_saturation_hamming_heuristic(
    graph: SymbolicAsyncGraph,
    vertices_hint: GraphColoredVertices,
) -> Vec<GraphColoredVertices> {
    let graph = graph.clone();
    let vertices_hint = vertices_hint.clone();

    let mut stack = vec![(graph, vertices_hint)];
    let mut ouput = Vec::<GraphColoredVertices>::new();

    while let Some((graph, vertices_hint)) = stack.pop() {
        assert!(!graph.unit_vertices().is_empty());

        let pivot_set = match vertices_hint.is_empty() {
            true => graph.unit_colored_vertices(),
            false => &vertices_hint,
        };
        let pivot = pivot_set.pick_singleton();

        assert!(!pivot.is_empty()); // trivially true; subgraph is nonempty (else returned above)

        let fwd_reachable = fwd_saturation(&graph, &pivot);

        let scc = bwd_saturation(&graph.restrict(&fwd_reachable), &pivot);

        let fwd_remaining = fwd_reachable.minus(&scc);
        if !fwd_remaining.is_empty() {
            let fwd_subgraph = graph.restrict(&fwd_remaining);

            let fwd_hint = pivot.ham_furthest_within(&fwd_remaining); // <-- the difference

            // chain_rec_saturation_hamming_heuristic(&fwd_subgraph, &fwd_hint, scc_dump);
            stack.push((fwd_subgraph, fwd_hint));
        }

        let rest_remaining = graph.unit_colored_vertices().minus(&fwd_reachable);
        if !rest_remaining.is_empty() {
            let rest_subgraph = graph.restrict(&rest_remaining);
            let rest_hint = graph.pre(&scc).intersect(&rest_remaining);
            // chain_rec_saturation_hamming_heuristic(&rest_subgraph, &rest_hint, scc_dump);
            stack.push((rest_subgraph, rest_hint));
        }

        // Output the scc.
        if !scc.is_singleton() {
            // todo this filter should probably be a part of a config/parameter
            ouput.push(scc);
        }
    }

    ouput
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
        F: Fn(SymbolicAsyncGraph) -> I,
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

        let scc_vec = decomposition_fn(async_graph).collect::<Vec<_>>();

        assert_eq!(scc_vec.len(), 2);

        // one of the components is { (a=false, b=false), (a=true, b=false) }
        assert!(scc_vec.contains(&b_false));
        // the other is { (a=false, b=true), (a=true, b=true) }
        assert!(scc_vec.contains(&b_true));
    }

    #[test]
    fn chain_test() {
        basic_decomposition(|graph| chain(graph, TrimLvl::None));
    }

    use super::TrimLvl;

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

        let chain_scc_set = chain(async_graph.clone(), TrimLvl::None).collect::<HashSet<_>>();
        let fwd_bwd_scc_set = fwd_bwd_scc_decomposition_naive(async_graph).collect::<HashSet<_>>();

        assert_eq!(chain_scc_set, fwd_bwd_scc_set);
    }

    fn compare_trimming<F, I>(model_path: &str, decomposition_fn: F)
    where
        F: Fn(SymbolicAsyncGraph, TrimLvl) -> I,
        I: Iterator<Item = GraphColoredVertices>,
    {
        let bn = BooleanNetwork::try_from_file(model_path).unwrap();
        let bn = bn.inline_constants(true, true);

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

        let sccs_no_trim = decomposition_fn(graph.clone(), TrimLvl::None).collect::<HashSet<_>>();
        let sccs_single_trim =
            decomposition_fn(graph.clone(), TrimLvl::StartOnly).collect::<HashSet<_>>();
        let sccs_full_trim = decomposition_fn(graph, TrimLvl::Full).collect::<HashSet<_>>();

        assert!(sccs_single_trim.is_subset(&sccs_no_trim));
        assert!(sccs_full_trim.is_subset(&sccs_single_trim))
    }

    fn compare_fn_with_fwd_bwd<F, I>(model_path: &str, decomposition_fn: F)
    where
        F: Fn(SymbolicAsyncGraph) -> I,
        I: Iterator<Item = GraphColoredVertices>,
    {
        let bn = BooleanNetwork::try_from_file(model_path).unwrap();
        let bn = bn.inline_constants(true, true);

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
        let fwd_bwd_scc_set =
            fwd_bwd_scc_decomposition_naive(graph.clone()).collect::<HashSet<_>>();

        println!(" >> Computing with {}.", std::any::type_name::<F>());
        let chain_scc_set = decomposition_fn(graph).collect::<HashSet<_>>();

        println!(" >> Found {} SCCs.", fwd_bwd_scc_set.len());

        assert_eq!(chain_scc_set, fwd_bwd_scc_set);
    }

    #[test_resources("./models/bbm-inputs-true/*.aeon")]
    fn compare_chain_fwd_bwd_selected(model_path: &str) {
        compare_fn_with_fwd_bwd(model_path, |graph| chain(graph, TrimLvl::None));
    }

    #[test_resources("./models/bbm-inputs-true/*.aeon")]
    fn compare_chain_saturation_fwd_bwd_selected(model_path: &str) {
        compare_fn_with_fwd_bwd(model_path, chain_saturation);
    }

    #[test_resources("./models/bbm-inputs-true/*.aeon")]
    fn compare_chain_saturation_hamming_heuristic_fwd_bwd_selected(model_path: &str) {
        compare_fn_with_fwd_bwd(model_path, chain_saturation_hamming_heuristic);
    }

    #[test_resources("./models/bbm-inputs-true/*.aeon")]
    fn compare_trimming_chain(model_path: &str) {
        compare_trimming(model_path, chain);
    }
}
