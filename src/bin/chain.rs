use biodivine_lib_param_bn::{symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork};
use cejn::chain::chain;

fn main() {
    let bn_path = std::env::args()
        .nth(1)
        .expect("args[1] should be the file path to the bn");

    let bn = BooleanNetwork::try_from_file(bn_path).unwrap();
    assert_eq!(bn.num_parameters(), 0);
    assert_eq!(bn.num_implicit_parameters(), 0);

    let graph = SymbolicAsyncGraph::new(&bn).unwrap();

    let _unused = chain(&graph); // just care about creating them
}
