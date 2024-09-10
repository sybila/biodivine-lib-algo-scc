use crate::chain::chain_saturation_trim;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::fixed_points::FixedPoints;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::trap_spaces::{
    NetworkColoredSpaces, NetworkSpaces, SymbolicSpaceContext,
};
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};
use log::debug;
use std::collections::HashSet;

pub fn trap_separated_components(
    bn: &BooleanNetwork,
    ctx: &SymbolicSpaceContext,
    graph: &SymbolicAsyncGraph,
) -> Vec<GraphColoredVertices> {
    debug!(target: "trap-scc", "Start SCC detection using trap space decomposition.");

    let all_spaces = ctx.mk_unit_colored_spaces(graph);

    let rev_bn = network_time_reversal(bn);
    let rev_graph = SymbolicAsyncGraph::with_space_context(&rev_bn, ctx).unwrap();

    let all_traps = all_trap_spaces(ctx, graph, &all_spaces);
    let all_traps_count = all_traps.exact_cardinality();

    debug!(target: "trap-scc", "Found {} normal trap spaces.", all_traps_count);

    let mut remaining = graph.mk_unit_colored_vertices();
    let mut results = Vec::new();
    for (i, trap_space) in all_traps.spaces().iter().enumerate() {
        let trap_space_bdd = ctx.mk_space(&trap_space);

        let subspaces_bdd = ctx.mk_sub_spaces(&trap_space_bdd);
        let subspaces = NetworkSpaces::new(subspaces_bdd, ctx);
        let subspaces = &all_spaces.intersect_spaces(&subspaces);
        let rev_traps = all_trap_spaces(ctx, &rev_graph, subspaces);
        let rev_trap_count = rev_traps.exact_cardinality();

        if !rev_traps.is_empty() {
            debug!(target: "trap-scc", "Found {} reverse trap spaces.", rev_trap_count);
            for (j, rev_trap_space) in rev_traps.spaces().iter().enumerate() {
                let space_states = graph.mk_subspace(&rev_trap_space.to_values());
                let space_states = remaining.intersect(&space_states);

                if space_states.is_empty() {
                    continue;
                }

                debug!(
                    target: "trap-scc",
                    "[rev][{}/{}] Processing reverse trap space ({} state(s)).",
                    j+1,
                    rev_trap_count,
                    space_states.exact_cardinality(),
                );

                let sub_graph = graph.restrict(&space_states);
                remaining = remaining.minus(&space_states);

                let mut space_scc_list = chain_saturation_trim(&sub_graph).collect::<Vec<_>>();
                let space_scc_count = space_scc_list.len();
                results.append(&mut space_scc_list);

                debug!(
                    target: "trap-scc",
                    "[rev][{}/{}] Found {} SCCs ({} total). Remaining states: {}.",
                    j + 1,
                    rev_trap_count,
                    space_scc_count,
                    results.len(),
                    remaining.exact_cardinality()
                );
            }
        }

        let space_states = graph.mk_subspace(&trap_space.to_values());
        let space_states = remaining.intersect(&space_states);

        if space_states.is_empty() {
            continue;
        }

        debug!(
            target: "trap-scc",
            "[{}/{}] Processing trap space ({} state(s)).",
            i+1,
            all_traps_count,
            space_states.exact_cardinality()
        );

        let sub_graph = graph.restrict(&space_states);
        remaining = remaining.minus(&space_states);

        let mut space_scc_list = chain_saturation_trim(&sub_graph).collect::<Vec<_>>();
        let space_scc_count = space_scc_list.len();
        results.append(&mut space_scc_list);

        debug!(
            target: "trap-scc",
            "[{}/{}] Found {} SCCs ({} total). Remaining states: {}.",
            i + 1,
            all_traps_count,
            space_scc_count,
            results.len(),
            remaining.exact_cardinality()
        );
    }

    results
}

pub fn network_time_reversal(bn: &BooleanNetwork) -> BooleanNetwork {
    let mut result = bn.clone();
    for var in result.variables() {
        let update = result.get_update_function(var).clone().unwrap();
        let not_var = FnUpdate::mk_not(FnUpdate::Var(var));
        let update = update.substitute_variable(var, &not_var);
        result
            .set_update_function(var, Some(FnUpdate::mk_not(update)))
            .unwrap();
    }
    result.infer_valid_graph().unwrap()
}

/// Similar to TrapSpaces::essential_symbolic, but returns *all* trap spaces, not just
/// the percolated ones.
pub fn all_trap_spaces(
    ctx: &SymbolicSpaceContext,
    graph: &SymbolicAsyncGraph,
    restriction: &NetworkColoredSpaces,
) -> NetworkColoredSpaces {
    let bdd_ctx = ctx.bdd_variable_set();

    // We always start with the restriction set, because it should carry the information
    // about valid encoding of spaces.
    let mut to_merge = vec![restriction.as_bdd().clone()];
    for var in graph.variables() {
        let update_bdd = graph.get_symbolic_fn_update(var);
        let not_update_bdd = update_bdd.not();

        let has_up_transition = ctx.mk_can_go_to_true(update_bdd);

        let has_down_transition = ctx.mk_can_go_to_true(&not_update_bdd);

        let true_var = ctx.get_positive_variable(var);
        let false_var = ctx.get_negative_variable(var);

        let is_trap_1 = has_up_transition.imp(&bdd_ctx.mk_var(true_var));
        let is_trap_2 = has_down_transition.imp(&bdd_ctx.mk_var(false_var));
        let is_trap = is_trap_1.and(&is_trap_2);

        //let is_essential_1 = bdd_ctx.mk_var(true_var).and(&bdd_ctx.mk_var(false_var));
        //let is_essential_2 = has_up_transition.and(&has_down_transition);
        //let is_essential = is_essential_1.imp(&is_essential_2);

        // This will work in next version of lib-bdd:
        // let is_trap = bdd!(bdd_ctx, (has_up_transition => true_var) & (has_down_transition => false_var));
        // let is_essential = bdd!(bdd_ctx, (true_var & false_var) => (has_up_transition & has_down_transition));

        //to_merge.push(is_trap.and(&is_essential));
        to_merge.push(is_trap);
    }

    let trap_spaces = FixedPoints::symbolic_merge(bdd_ctx, to_merge, HashSet::default());
    

    NetworkColoredSpaces::new(trap_spaces, ctx)
}
