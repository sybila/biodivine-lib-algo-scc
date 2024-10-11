use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;
use cejn::transients::is_universally_transient;

fn main() {
    env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 2);

    let bn = BooleanNetwork::try_from_file(&args[1]).unwrap();
    let bn = bn.inline_constants(true, true);
    let graph = SymbolicAsyncGraph::new(&bn).unwrap();

    println!("Loaded BN with {} variables.", bn.num_vars());

    let mut scc_list = cejn::chain::chain_saturation_trim(&graph).collect::<Vec<_>>();
    scc_list.sort_by_key(|it| it.exact_cardinality());

    let long_lived = scc_list
        .iter()
        .filter(|it| !is_universally_transient(&graph, it))
        .count();
    println!("all_non_trivial_scc, long_lived_scc");
    println!("{}, {}", scc_list.len(), long_lived);
}
