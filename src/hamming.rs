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

        let res = max_dist_bad(choice_set, &self_singleton_valuation);
        assert!(res.is_singleton());

        res
    }
}

fn max_dist_bad(
    choice_set: &GraphColoredVertices,
    pivot_singleton_valuation: &BddValuation,
) -> GraphColoredVertices {
    // let mut result_valuation = BddPartialValuation::from(pivot_singleton_valuation.clone()); // todo maybe `Partial...` not needed

    let (_, result_valuation) = rec_max_dist_bad(
        choice_set.as_bdd().root_pointer(),
        choice_set.vertices().as_bdd(),
        pivot_singleton_valuation,
    )
    .expect("a valid assignment should be found -> proper distance should be returned");

    let result_valuation_bdd = Bdd::from(result_valuation);
    println!(
        "result valuation: {:?}",
        result_valuation_bdd.first_valuation()
    ); // todo debug print -> remove

    let res = choice_set.copy(result_valuation_bdd.clone());

    assert_eq!(res.as_bdd().to_owned(), result_valuation_bdd); // todo debug assert -> remove

    assert!(res.is_singleton());

    res
}

// todo do not use recursive alg
fn rec_max_dist_bad(
    curr_choice_set_node_ptr: BddPointer,
    choice_set: &Bdd,
    pivot_singleton_valuation: &BddValuation,
) -> Option<(usize, BddValuation)> {
    if curr_choice_set_node_ptr.is_zero() {
        return None; // this path (valuation) does not belong to the choice set
    }

    if curr_choice_set_node_ptr.is_one() {
        // todo this clone increases the complexity -> use "caches" (two valuations as &mut params) or just push to raw Vec<bool>
        let mut valuation = pivot_singleton_valuation.clone();
        for idx in 0..choice_set.num_vars() {
            let fixed_value = true; // arbitrary choice; the point is that it *has some* fixed value
            valuation.set_value(BddVariable::from_index(idx as usize), fixed_value);
        }

        return Some((0, valuation));
    }

    let low_child_ptr = choice_set.low_link_of(curr_choice_set_node_ptr);
    let high_child_ptr = choice_set.high_link_of(curr_choice_set_node_ptr);

    let this_node_var = choice_set.var_of(curr_choice_set_node_ptr);
    let low_child_var = choice_set.var_of(low_child_ptr);
    let high_child_var = choice_set.var_of(high_child_ptr);

    let mut low_child = rec_max_dist_bad(low_child_ptr, choice_set, pivot_singleton_valuation);
    let mut high_child = rec_max_dist_bad(high_child_ptr, choice_set, pivot_singleton_valuation);

    low_child
        .iter_mut()
        .for_each(|(low_child_distance, low_child_valuation)| {
            for idx in (this_node_var.to_index() + 1)..low_child_var.to_index() {
                let fixed_value = false; // arbitrary choice; the point is that it *has some* fixed value
                low_child_valuation.set_value(BddVariable::from_index(idx), fixed_value);
                *low_child_distance += 1;
            }
        });

    high_child
        .iter_mut()
        .for_each(|(high_child_distance, high_child_valuation)| {
            for idx in (this_node_var.to_index() + 1)..high_child_var.to_index() {
                let fixed_value = false; // arbitrary choice; the point is that it *has some* fixed value
                high_child_valuation.set_value(BddVariable::from_index(idx), fixed_value);
                *high_child_distance += 1;
            }
        });

    if pivot_singleton_valuation.value(choice_set.var_of(curr_choice_set_node_ptr)) {
        low_child.iter_mut().for_each(|(distance, _)| {
            *distance += 1;
        });
    } else {
        high_child.iter_mut().for_each(|(distance, _)| {
            *distance += 1;
        });
    }

    match (low_child, high_child) {
        (
            Some((low_child_distance, low_child_valuation)),
            Some((high_child_distance, high_child_valuation)),
        ) => {
            println!("--------------------- in the both some branch");
            // prefer greater distance
            if low_child_distance < high_child_distance {
                let high_child_valuation = {
                    let mut valuation = high_child_valuation;
                    valuation.set_value(this_node_var, true);
                    valuation
                };
                Some((high_child_distance, high_child_valuation))
            } else {
                let low_child_valuation = {
                    let mut valuation = low_child_valuation;
                    valuation.set_value(this_node_var, false);
                    valuation
                };
                Some((low_child_distance, low_child_valuation))
            }
        }
        (Some((low_hild_distance, low_child_valuation)), None) => {
            let low_child_valuation = {
                let mut valuation = low_child_valuation;
                valuation.set_value(this_node_var, false);
                valuation
            };
            Some((low_hild_distance, low_child_valuation))
        }
        (None, Some((high_child_distance, high_child_valuation))) => {
            let high_child_valuation = {
                let mut valuation = high_child_valuation;
                valuation.set_value(this_node_var, true);
                valuation
            };
            Some((high_child_distance, high_child_valuation))
        }
        (None, None) => unreachable!(),
    }
}

// fn max_dist(
//     choice_set: &GraphColoredVertices,
//     pivot_singleton_valuation: &BddValuation,
// ) -> GraphColoredVertices {

// fn rec_max_dist(
//     curr_choice_set_node_ptr: BddPointer,
//     choice_set: &Bdd,
//     pivot_singleton_valuation: &BddValuation,
// ) -> Option<(usize, BddValuation)> {
// }

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
