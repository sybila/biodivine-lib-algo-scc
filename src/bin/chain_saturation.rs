use biodivine_lib_algo_scc::chain::Config;
use biodivine_lib_algo_scc::chain::Strategy;
use biodivine_lib_algo_scc::chain::chain;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 2);

    let bn = BooleanNetwork::try_from_file(&args[1]).unwrap();
    let bn = bn.inline_constants(true, true);
    let graph = SymbolicAsyncGraph::new(&bn).unwrap();

    println!("Loaded BN with {} variables.", bn.num_vars());

    let mut scc_list = chain(
        graph,
        Config {
            strategy: Strategy::Saturation,
            ..Default::default()
        },
    )
    .collect::<Vec<_>>();
    scc_list.sort_by_key(|it| it.exact_cardinality());

    let trivial = scc_list.iter().filter(|it| it.is_singleton()).count();

    println!("all_scc, trivial_scc, sizes...");
    print!("{}, {}", scc_list.len(), trivial);
    for scc in scc_list.iter().rev().take(100) {
        if !scc.is_singleton() {
            print!(", {}", scc.exact_cardinality());
        }
    }
    println!();
}
