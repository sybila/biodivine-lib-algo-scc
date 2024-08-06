use biodivine_lib_bdd::{Bdd, BddPointer, BddValuation, BddVariable};
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set, symbolic_async_graph::GraphColoredVertices,
};

pub trait Hamming {
    fn ham_furthest_within(&self, choice_set: &GraphColoredVertices) -> GraphColoredVertices;
}

impl Hamming for GraphColoredVertices {
    fn ham_furthest_within(&self, choice_set: &GraphColoredVertices) -> GraphColoredVertices {
        assert!(!choice_set.is_empty());
        assert!(self.is_singleton());

        let self_singleton_valuation = self.vertices().as_bdd().sat_witness().unwrap();

        let res = max_dist(choice_set, &self_singleton_valuation);
        assert!(res.is_singleton());

        res
    }
}

fn max_dist(
    choice_set: &GraphColoredVertices,
    pivot_singleton_valuation: &BddValuation,
) -> GraphColoredVertices {
    let choice_set_bdd = choice_set.vertices().as_bdd().clone();

    let mut chosen_path_cache =
        vec![None::<(usize, ChosenChild)>; choice_set_bdd.clone().to_nodes().len()];

    rec_max_dist_path(
        choice_set_bdd.root_pointer(),
        &choice_set_bdd,
        pivot_singleton_valuation,
        &mut chosen_path_cache,
    );

    let chosen_path_cache = chosen_path_cache;

    let mut valuation = pivot_singleton_valuation.clone();
    let mut curr_var_idx = 0;
    let mut next_known_var_idx = if choice_set_bdd.root_pointer().is_one() {
        // get the total count of the variables in the graph (weird way to do so)
        pivot_singleton_valuation.to_values().len()
    } else {
        // get the root-variable's index *within the valuation* (ie the roots variable id)
        let variable_0 = choice_set_bdd.var_of(choice_set_bdd.root_pointer());
        variable_0.to_index()
    };
    let mut next_node_ptr = choice_set_bdd.root_pointer();

    loop {
        while curr_var_idx < next_known_var_idx {
            // the path within the bdd does not say anything about this current var -> set it to the negation of the pivot valuation
            let negated_value =
                !pivot_singleton_valuation.value(BddVariable::from_index(curr_var_idx));
            valuation.set_value(BddVariable::from_index(curr_var_idx), negated_value);

            curr_var_idx += 1;
        }

        // curr_var_idx == next_known_var_idx -> read next step from the cache

        match chosen_path_cache[next_node_ptr.to_index()]
            .expect("should be set")
            .1
        {
            ChosenChild::Low => {
                valuation.set_value(BddVariable::from_index(curr_var_idx), false);
                next_node_ptr = choice_set_bdd.low_link_of(next_node_ptr);
                next_known_var_idx = choice_set_bdd.var_of(next_node_ptr).to_index();
            }
            ChosenChild::High => {
                valuation.set_value(BddVariable::from_index(curr_var_idx), true);
                next_node_ptr = choice_set_bdd.high_link_of(next_node_ptr);
                next_known_var_idx = choice_set_bdd.var_of(next_node_ptr).to_index();
            }
            ChosenChild::NoChildAvailable => {
                // leaf node reached
                break;
            }
        }

        curr_var_idx += 1;
    }

    let res = choice_set.copy(Bdd::from(valuation));

    assert!(res.is_singleton());

    res
}

#[derive(Clone, Copy, Debug)]
enum ChosenChild {
    Low,
    High,
    NoChildAvailable,
}

/// traverses the choice_set to find the "most distant" valuation possible
/// does not update the valuation in the root if there are variables missing "above it"
fn rec_max_dist_path(
    curr_choice_set_node_ptr: BddPointer,
    choice_set: &Bdd,
    pivot_singleton_valuation: &BddValuation,
    chosen_path_cache: &mut Vec<Option<(usize, ChosenChild)>>,
) {
    if chosen_path_cache[curr_choice_set_node_ptr.to_index()].is_some() {
        return; // already "solved"
    }

    if curr_choice_set_node_ptr.is_zero() {
        // should already be set to `None` - do this just to be sure
        chosen_path_cache[curr_choice_set_node_ptr.to_index()] = None;
        return;
    }

    if curr_choice_set_node_ptr.is_one() {
        // signal that this is branch should be considered -> set to Some
        chosen_path_cache[curr_choice_set_node_ptr.to_index()] =
            Some((0 /* no distance */, ChosenChild::NoChildAvailable));
        return;
    }

    let low_child_ptr = choice_set.low_link_of(curr_choice_set_node_ptr);
    let high_child_ptr = choice_set.high_link_of(curr_choice_set_node_ptr);

    // ensure children's caches computed
    rec_max_dist_path(
        low_child_ptr,
        choice_set,
        pivot_singleton_valuation,
        chosen_path_cache,
    );
    rec_max_dist_path(
        high_child_ptr,
        choice_set,
        pivot_singleton_valuation,
        chosen_path_cache,
    );

    match (
        &chosen_path_cache[low_child_ptr.to_index()],
        &chosen_path_cache[high_child_ptr.to_index()],
    ) {
        (Some((low_child_dist, _)), Some((high_child_dist, _))) => {
            let choose_low_dist_increase =
                if pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
                    // notice the absence of negation - want neq
                    1
                } else {
                    0
                };
            let choose_high_dist_increase =
                if !pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
                    // notice the presence of negation - want neq
                    1
                } else {
                    0
                };

            let choose_low_skipped_vars_dist = choice_set
                .var_of(low_child_ptr)
                .to_index()
                // subtract or 0 if rhs is larger
                .saturating_sub(choice_set.var_of(curr_choice_set_node_ptr).to_index() + 1);
            let choose_high_skipped_vars_dist = choice_set
                .var_of(high_child_ptr)
                .to_index()
                // subtract or 0 if rhs is larger
                .saturating_sub(choice_set.var_of(curr_choice_set_node_ptr).to_index() + 1);

            let this_dist_low =
                *low_child_dist + choose_low_dist_increase + choose_low_skipped_vars_dist;
            let this_dist_high =
                *high_child_dist + choose_high_dist_increase + choose_high_skipped_vars_dist;

            if this_dist_low < this_dist_high {
                chosen_path_cache[curr_choice_set_node_ptr.to_index()] =
                    Some((this_dist_high, ChosenChild::High));
            } else {
                chosen_path_cache[curr_choice_set_node_ptr.to_index()] =
                    Some((this_dist_low, ChosenChild::Low));
            }
        }

        (Some((low_child_dist, _)), None) => {
            let choose_low_dist_increase =
                if pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
                    // notice the absence of negation - want neq
                    1
                } else {
                    0
                };
            let choose_low_skipped_vars_dist = choice_set
                .var_of(low_child_ptr)
                .to_index()
                // subtract or 0 if rhs is larger
                .saturating_sub(choice_set.var_of(curr_choice_set_node_ptr).to_index() + 1);

            let this_dist_low =
                *low_child_dist + choose_low_dist_increase + choose_low_skipped_vars_dist;

            chosen_path_cache[curr_choice_set_node_ptr.to_index()] =
                Some((this_dist_low, ChosenChild::Low));
        }

        (None, Some((high_child_dist, _))) => {
            let choose_high_dist_increase =
                if !pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
                    // notice the presence of negation - want neq
                    1
                } else {
                    0
                };

            let choose_high_skipped_vars_dist = choice_set
                .var_of(high_child_ptr)
                .to_index()
                // subtract or 0 if rhs is larger
                .saturating_sub(choice_set.var_of(curr_choice_set_node_ptr).to_index() + 1);

            let this_dist_high =
                *high_child_dist + choose_high_dist_increase + choose_high_skipped_vars_dist;

            chosen_path_cache[curr_choice_set_node_ptr.to_index()] =
                Some((this_dist_high, ChosenChild::High));
        }

        (None, None) => {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use biodivine_lib_param_bn::{
        biodivine_std::traits::Set, symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork,
    };

    use crate::hamming::Hamming;

    fn basic_async_graph() -> SymbolicAsyncGraph {
        let bool_network = BooleanNetwork::try_from(
            r#"
            A -| A
            B -> B
            $A: !A
            $B: B
            "#,
        ) // the exact network does not matter; just the presence of the states
        .unwrap();
        SymbolicAsyncGraph::new(&bool_network).unwrap()
    }

    #[test]
    fn test_ham_furthest_within() {
        let async_graph = basic_async_graph();

        let (var_a, var_b) = {
            let mut vars = async_graph.variables();
            let var_a = vars.next().unwrap();
            let var_b = vars.next().unwrap();
            assert!(vars.next().is_none());
            (var_a, var_b)
        };

        let unit_set = async_graph.unit_colored_vertices().to_owned();

        let a_true = unit_set.fix_network_variable(var_a, true);
        let b_true = unit_set.fix_network_variable(var_b, true);
        let a_false = unit_set.fix_network_variable(var_a, false);
        let b_false = unit_set.fix_network_variable(var_b, false);

        let false_false = a_false.intersect(&b_false);
        let false_true = a_false.intersect(&b_true);
        let true_false = a_true.intersect(&b_false);
        let true_true = a_true.intersect(&b_true);

        // must choose the single one
        assert_eq!(false_false.ham_furthest_within(&true_true), true_true);
        assert_eq!(false_false.ham_furthest_within(&true_false), true_false);
        assert_eq!(false_false.ham_furthest_within(&false_true), false_true);
        assert_eq!(false_false.ham_furthest_within(&false_false), false_false);

        // must choose the most distant one - all negated
        assert_eq!(false_false.ham_furthest_within(&unit_set), true_true);
        assert_eq!(false_true.ham_furthest_within(&unit_set), true_false);
        assert_eq!(true_false.ham_furthest_within(&unit_set), false_true);
        assert_eq!(true_true.ham_furthest_within(&unit_set), false_false);
    }

    #[test]
    fn test_unit_set() {
        let async_graph = basic_async_graph();

        let valuations = async_graph
            .unit_colored_vertices()
            .as_bdd()
            .sat_valuations()
            .collect::<Vec<_>>();

        println!("{:?}", valuations);
    }
}
