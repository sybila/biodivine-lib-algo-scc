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

        max_dist(choice_set, &self_singleton_valuation)
    }
}

fn max_dist(
    choice_set: &GraphColoredVertices,
    pivot_singleton_valuation: &BddValuation,
) -> GraphColoredVertices {
    // let mut result_valuation = BddPartialValuation::from(pivot_singleton_valuation.clone()); // todo maybe `Partial...` not needed

    let (_, result_valuation) = rec_max_dist(
        choice_set.as_bdd().root_pointer(),
        choice_set.vertices().as_bdd(),
        pivot_singleton_valuation,
    )
    .expect("a valid assignment should be found -> proper distance should be returned");

    let res = choice_set.copy(Bdd::from(result_valuation));

    assert!(res.is_singleton());

    res
}

// todo do not use recursive alg
fn rec_max_dist(
    curr_choice_set_node_ptr: BddPointer,
    choice_set: &Bdd,
    pivot_singleton_valuation: &BddValuation,
) -> Option<(usize, BddValuation)> {
    if curr_choice_set_node_ptr.is_zero() {
        return None; // this path (valuation) does not belong to the choice set
    }

    if curr_choice_set_node_ptr.is_one() {
        // todo this clone increases the complexity -> use "caches" (two valuations as &mut params) or just push to raw Vec<bool>
        let valuation = pivot_singleton_valuation.clone();
        return Some((0, valuation));
    }

    let low_child_ptr = choice_set.low_link_of(curr_choice_set_node_ptr);
    let high_child_ptr = choice_set.high_link_of(curr_choice_set_node_ptr);

    let this_node_var = choice_set.var_of(curr_choice_set_node_ptr);
    let low_child_var = choice_set.var_of(low_child_ptr);
    let high_child_var = choice_set.var_of(high_child_ptr);

    let mut low_child = rec_max_dist(low_child_ptr, choice_set, pivot_singleton_valuation);
    let mut high_child = rec_max_dist(high_child_ptr, choice_set, pivot_singleton_valuation);

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
        high_child.iter_mut().for_each(|(distance, _)| {
            *distance += 1;
        });
    } else {
        low_child.iter_mut().for_each(|(distance, _)| {
            *distance += 1;
        });
    }

    if let (Some((low_child_dist, _)), Some((high_child_dist, _))) = (&low_child, &high_child) {
        if *low_child_dist < *high_child_dist {
            high_child
        } else {
            low_child
        }
    } else {
        low_child.or(high_child)
    }
}
