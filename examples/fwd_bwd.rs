use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 2);

    let bn = BooleanNetwork::try_from_file(&args[1]).unwrap();
    let graph = SymbolicAsyncGraph::new(&bn).unwrap();

    println!("Loaded BN with {} variables.", bn.num_vars());

    let mut scc_list = cejn::fwd_bwd::fwd_bwd_scc_decomposition(&graph).collect::<Vec<_>>();
    scc_list.sort_by_key(|it| it.exact_cardinality());

    let trivial = scc_list.iter().filter(|it| it.is_singleton()).count();

    println!("all_scc, trivial_scc, sizes...");
    print!("{}, {}", scc_list.len(), trivial);
    for scc in scc_list.iter() {
        if !scc.is_singleton() {
            print!(", {}", scc.exact_cardinality());
        }
    }
    println!();
}
