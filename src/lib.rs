#[allow(dead_code)]
mod chain;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use biodivine_lib_param_bn::{
        biodivine_std::traits::Set, symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork,
        Monotonicity, Regulation, RegulatoryGraph,
    };

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn lib_param_bn_tryout() -> Result<(), String> {
        let mut rg = RegulatoryGraph::new(vec!["A".into(), "B".into(), "C".into()]);
        rg.add_raw_regulation(Regulation {
            regulator: rg.find_variable("A").unwrap(),
            target: rg.find_variable("B").unwrap(),
            observable: true,
            monotonicity: Some(Monotonicity::Activation),
        })?;
        rg.add_raw_regulation(Regulation {
            regulator: rg.find_variable("B").unwrap(),
            target: rg.find_variable("C").unwrap(),
            observable: true,
            monotonicity: Some(Monotonicity::Activation),
        })?;
        rg.add_raw_regulation(Regulation {
            regulator: rg.find_variable("C").unwrap(),
            target: rg.find_variable("A").unwrap(),
            observable: true,
            monotonicity: Some(Monotonicity::Activation),
        })?;
        rg.add_raw_regulation(Regulation {
            regulator: rg.find_variable("C").unwrap(),
            target: rg.find_variable("B").unwrap(),
            observable: false,
            monotonicity: Some(Monotonicity::Inhibition),
        })?;

        let some_id = rg.find_variable("A").unwrap();
        let xd = &rg[some_id];

        println!("{:?}", xd);

        Ok(())
    }

    #[test]
    fn lib_param_bn_tryout_2() -> Result<(), String> {
        let bool_network = BooleanNetwork::try_from(
            r"
        A -> B
        C -|? B
        # Update function of variable B:
        $B: A
        C -> A
        B -> A
        A -| A
        $A: C | f(A, B)
    ",
        )?;

        let trans_graph = SymbolicAsyncGraph::new(&bool_network)?;

        let id_b = bool_network.as_graph().find_variable("B").unwrap();
        let b_is_true = trans_graph.fix_network_variable(id_b, true);
        let b_is_false = trans_graph.fix_network_variable(id_b, false);

        let empty_subgraph = b_is_true.intersect_vertices(&b_is_false.vertices());

        assert!(empty_subgraph.is_empty());

        Ok(())
    }
}
