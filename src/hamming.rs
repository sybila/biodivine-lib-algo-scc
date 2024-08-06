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
    let mut valuation_cache = {
        let mut cache = Vec::<Option<(usize, Vec<bool>)>>::new(); // keep in mind the valuation is reversed
        for _ in 0..choice_set.vertices().as_bdd().clone().to_nodes().len() {
            cache.push(None);
        }

        cache
    };

    rec_max_dist(
        choice_set.as_bdd().root_pointer(),
        choice_set.vertices().as_bdd(),
        pivot_singleton_valuation,
        &mut valuation_cache,
    );

    let (_, mut unsanitized_result_valuation) = valuation_cache
        [choice_set.vertices().as_bdd().root_pointer().to_index()]
    .clone()
    .expect("cache of the root should be filled now");

    let to_be_filled_up_to_idx = if choice_set.as_bdd().root_pointer().is_one() {
        // get the total count of the variables in the graph (weird way to do so)
        pivot_singleton_valuation.to_values().len()
    } else {
        // get the root-variable's index *within the valuation* (ie the roots variable id)
        let variable_0 = choice_set
            .as_bdd()
            .var_of(choice_set.as_bdd().root_pointer());
        variable_0.to_index()
    };

    for idx in (0..to_be_filled_up_to_idx).rev() {
        let negated_value = !pivot_singleton_valuation.value(BddVariable::from_index(idx));
        unsanitized_result_valuation.push(negated_value);
    }

    let result_valuation_vec = unsanitized_result_valuation;
    let mut actual_result_valuation = pivot_singleton_valuation.clone();
    for (idx, value) in result_valuation_vec.into_iter().rev().enumerate() {
        actual_result_valuation.set_value(BddVariable::from_index(idx), value);
    }

    let res = choice_set.copy(Bdd::from(actual_result_valuation));

    assert!(res.is_singleton());

    res
}

/// traverses the choice_set to find the "most distant" valuation possible
/// does not update the valuation in the root if there are variables missing "above it"
fn rec_max_dist(
    curr_choice_set_node_ptr: BddPointer,
    choice_set: &Bdd,
    pivot_singleton_valuation: &BddValuation,
    // todo could avoid using `Vec<_>`s altogether - just keep the "chosen child" in the cache -> do not forget about the skipped variables
    valuation_cache: &mut Vec<Option<(usize, Vec<bool>)>>,
) {
    if valuation_cache[curr_choice_set_node_ptr.to_index()].is_some() {
        return; // already "solved"
    }

    if curr_choice_set_node_ptr.is_zero() {
        // should already be set to `None` - do this just to be sure
        valuation_cache[curr_choice_set_node_ptr.to_index()] = None;
        return;
    }

    if curr_choice_set_node_ptr.is_one() {
        // signal that this is branch should be considered -> set to Some
        valuation_cache[curr_choice_set_node_ptr.to_index()] =
            Some((0 /* no distance */, Vec::new()));
        return;
    }

    let low_child_ptr = choice_set.low_link_of(curr_choice_set_node_ptr);
    let high_child_ptr = choice_set.high_link_of(curr_choice_set_node_ptr);

    // ensure children's caches computed
    rec_max_dist(
        low_child_ptr,
        choice_set,
        pivot_singleton_valuation,
        valuation_cache,
    );
    rec_max_dist(
        high_child_ptr,
        choice_set,
        pivot_singleton_valuation,
        valuation_cache,
    );

    match (
        &valuation_cache[low_child_ptr.to_index()],
        &valuation_cache[high_child_ptr.to_index()],
    ) {
        (Some((low_child_dist, low_child_val)), Some((high_child_dist, high_child_val))) => {
            let mut this_valuation_high = high_child_val.clone();
            let mut this_dist_high = *high_child_dist;

            if !pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
                // notice the presence of negation - want neq
                this_dist_high += 1;
            }

            for idx in ((choice_set.var_of(curr_choice_set_node_ptr).to_index() + 1)
                ..choice_set.var_of(high_child_ptr).to_index())
                .rev()
            {
                let negated_value = !pivot_singleton_valuation.value(BddVariable::from_index(idx));
                this_valuation_high.push(negated_value);
                this_dist_high += 1;
            }
            this_valuation_high.push(true);

            let mut this_valuation_low = low_child_val.clone();
            let mut this_dist_low = *low_child_dist;

            if pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
                // notice the absence of negation - want neq
                this_dist_low += 1;
            }

            for idx in ((choice_set.var_of(curr_choice_set_node_ptr).to_index() + 1)
                ..choice_set.var_of(low_child_ptr).to_index())
                .rev()
            {
                let negated_value = !pivot_singleton_valuation.value(BddVariable::from_index(idx));
                this_valuation_low.push(negated_value);
                this_dist_low += 1;
            }
            this_valuation_low.push(false);

            if this_dist_low < this_dist_high {
                valuation_cache[curr_choice_set_node_ptr.to_index()] =
                    // (this_valuation_high, Some(this_dist_high));
                    Some((this_dist_high, this_valuation_high));
            } else {
                valuation_cache[curr_choice_set_node_ptr.to_index()] =
                    // (this_valuation_low, Some(this_dist_low));
                    Some((this_dist_low, this_valuation_low));
            }
        }

        (Some((low_child_dist, low_child_val)), None) => {
            let mut this_valuation = low_child_val.clone();
            let mut this_dist = *low_child_dist;

            if pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
                // notice the absence of negation - want neq
                this_dist += 1;
            }

            for idx in ((choice_set.var_of(curr_choice_set_node_ptr).to_index() + 1)
                ..choice_set.var_of(low_child_ptr).to_index())
                .rev()
            {
                let negated_value = !pivot_singleton_valuation.value(BddVariable::from_index(idx));
                this_valuation.push(negated_value);
                this_dist += 1;
            }
            this_valuation.push(false);

            valuation_cache[curr_choice_set_node_ptr.to_index()] =
                Some((this_dist, this_valuation));
        }

        (None, Some((high_child_dist, high_child_val))) => {
            let mut this_valuation = high_child_val.clone();
            let mut this_dist = *high_child_dist;

            if !pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
                // notice the presence of negation - want neq
                this_dist += 1;
            }

            for idx in ((choice_set.var_of(curr_choice_set_node_ptr).to_index() + 1)
                ..choice_set.var_of(high_child_ptr).to_index())
                .rev()
            {
                let negated_value = !pivot_singleton_valuation.value(BddVariable::from_index(idx));
                this_valuation.push(negated_value);
                this_dist += 1;
            }
            this_valuation.push(true);

            valuation_cache[curr_choice_set_node_ptr.to_index()] =
                // (this_valuation, Some(this_dist));
                Some((this_dist, this_valuation));
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
