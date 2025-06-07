use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::fixed_points::FixedPoints;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;
use cejn::transients::{enclosing_subspace, is_long_lived, is_subspace, is_trapped};

fn main() {
    env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 2);

    let bn = BooleanNetwork::try_from_file(&args[1]).unwrap();
    let bn = bn.inline_constants(true, true);
    let graph = SymbolicAsyncGraph::new(&bn).unwrap();

    println!("Loaded BN with {} variables.", bn.num_vars());

    let mut scc_list = cejn::chain::chain_saturation_trim_long_lived(&graph).collect::<Vec<_>>();
    scc_list.sort_by_key(|it| it.exact_cardinality());

    let fixed_points = FixedPoints::symbolic(&graph, graph.unit_colored_vertices());

    let complex_attractors = scc_list
        .iter()
        .filter(|it| is_trapped(&graph, it))
        .cloned()
        .collect::<Vec<_>>();

    let long_lived_transients = scc_list
        .iter()
        .filter(|it| is_long_lived(&graph, it) && !is_trapped(&graph, it))
        .cloned()
        .collect::<Vec<_>>();

    let mut attractor_phenotypes = Vec::new();
    let mut weak_basins = Vec::new();

    for attr in &complex_attractors {
        weak_basins.push(graph.reach_backward(attr));
        attractor_phenotypes.push(enclosing_subspace(&graph, &attr.vertices()));
    }

    for fix in fixed_points.vertices() {
        let fix = bn.variables().zip(fix.values()).collect::<Vec<_>>();
        let fix_set = graph.mk_subspace(&fix);
        weak_basins.push(graph.reach_backward(&fix_set));
        attractor_phenotypes.push(enclosing_subspace(&graph, &fix_set.vertices()));
    }

    let mut strong_basins = Vec::new();

    for weak_basin in &weak_basins {
        let mut strong_basin = weak_basin.clone();
        for other_basin in &weak_basins {
            if other_basin == weak_basin {
                continue;
            }
            strong_basin = strong_basin.minus(other_basin);
        }
        strong_basins.push(strong_basin);
    }

    let mut has_weak_decision = false;
    let mut has_strong_decision = false;
    let mut has_unique_phenotype = false;

    for ltt in &long_lived_transients {
        println!(
            "Found a long Lived transient; size {}",
            ltt.exact_cardinality()
        );
        let intersecting_wb = weak_basins
            .iter()
            .enumerate()
            .filter(|(_, it)| ltt.is_subset(it))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let intersecting_sb = strong_basins
            .iter()
            .enumerate()
            .filter(|(_, it)| ltt.is_subset(it))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        println!("Intersecting weak basins: {:?}", intersecting_wb);
        println!("Intersecting strong basins: {:?}", intersecting_sb);

        if !intersecting_sb.is_empty() {
            println!("Found LLT within a strong basin.")
        }

        let ltt_successors = graph.post(ltt);
        let successor_sb = strong_basins
            .iter()
            .enumerate()
            .filter(|(i, it)| {
                !ltt_successors.intersect(it).is_empty() && !intersecting_sb.contains(i)
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        println!("One-step reachable strong basins: {:?}", successor_sb);
        // There is a strong basin that we can get into using this LLT.
        has_weak_decision = has_weak_decision || !successor_sb.is_empty();

        // TODO: We want to restrict this to the smallest enclosing trap space (SD node)?
        let ltt_basin = graph.reach_backward(ltt);
        let ltt_basin_minus_ltt = ltt_basin.minus(ltt);
        let basin_successors = graph.post(&ltt_basin_minus_ltt).minus(ltt);
        let basin_successor_wb = weak_basins
            .iter()
            .enumerate()
            .filter(|(_, it)| !basin_successors.intersect(it).is_empty())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        println!(
            "Basins one-step reachable from LLT basin: {:?}",
            basin_successor_wb
        );

        let ltt_subspace = enclosing_subspace(&graph, &ltt.vertices());

        println!(
            "Fixed variables: {}/{}",
            ltt_subspace.count_fixed(),
            bn.num_vars()
        );

        for i in &intersecting_wb {
            if !basin_successor_wb.contains(i) {
                // Being a strong decision point means that there is an attractor that is reachable
                // from the LLT, but isn't reachable from the basin of the LLT once LLT is removed.
                println!("Found LLT that is a strong decision point.");
                has_strong_decision = true;
                for (var, value) in ltt_subspace.to_values() {
                    println!("\t{}:{}", bn.get_variable_name(var), value)
                }
                break;
            }
        }

        // List of phenotypes that are the sub-spaces of this LLT phenotype. If the LLT does not
        // have any "sub-phenotypes", it is unique.
        let inner_phenotypes = attractor_phenotypes
            .iter()
            .enumerate()
            .filter(|(_, it)| is_subspace(it, &ltt_subspace))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        has_unique_phenotype = has_unique_phenotype || inner_phenotypes.is_empty();

        println!("Inner phenotypes: {:?}", inner_phenotypes);
        if inner_phenotypes.is_empty() {
            println!("LLT is a unique phenotype!")
        }
    }

    println!("long_lived_transients, complex_attractors, fixed_points, has_weak_decision, has_strong_decision, has_unique_phenotype");
    println!(
        "{}, {}, {}, {}, {}, {}",
        long_lived_transients.len(),
        complex_attractors.len(),
        fixed_points.exact_cardinality(),
        has_weak_decision,
        has_strong_decision,
        has_unique_phenotype,
    );
}
