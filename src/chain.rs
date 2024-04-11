use std::collections::HashMap;

use biodivine_lib_bdd::Bdd;

struct Graph {
    /// List of all variables in the graph.
    /// The order of variables is important for the update functions.
    /// Furthermore, the order of variables corresponds to the order they occur in the BDDs.
    sorted_variables: Vec<String>,
    /// Update functions describing the edges of the graph.
    /// The order of the functions corresponds to the order of variables.
    sorted_update_functions: Vec<Bdd>,
    /// Mapping from variable names to their index in the `sorted_variables` list.
    mapper: HashMap<String, usize>,
}

impl Graph {
    /// Gets the update function of the variable with the given name.
    ///
    /// # Panics
    ///
    /// Panics if the variable with the given name does not exist.
    fn update_function_of(&self, variable_name: &str) -> &Bdd {
        let uf_idx = self.mapper.get(variable_name).expect("unknown variable");

        &self.sorted_update_functions[*uf_idx]
    }
}

fn chain() {
    todo!();
}
