// Dependency graph
// Used to compute dependencies between top level declarations

// Maybe in the future

use std::collections::{HashMap, HashSet};

pub struct DependencyGraph {
    pub graph: HashMap<String, HashSet<String>>
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new()
        }
    }

    pub fn add_dependency(&mut self, from: String, to: String) {
        self.graph.entry(from).or_insert(HashSet::new()).insert(to);
    }

    pub fn get_dependencies(&self, from: &str) -> Option<&HashSet<String>> {
        self.graph.get(from)
    }

    pub fn compute_dependencies(&self, from: &str) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        let mut visited = HashSet::new();
        self.compute_dependencies_recursive(from, &mut dependencies, &mut visited);
        dependencies
    }

    fn compute_dependencies_recursive(&self, from: &str, dependencies: &mut HashSet<String>, visited: &mut HashSet<String>) {
        if visited.contains(from) {
            return;
        }

        visited.insert(from.to_string());

        if let Some(deps) = self.get_dependencies(from) {
            for dep in deps {
                dependencies.insert(dep.to_string());
                self.compute_dependencies_recursive(dep, dependencies, visited);
            }
        }
    }

    pub fn print_graph(&self) {
        for (from, to) in &self.graph {
            println!("{} -> {:?}", from, to);
        }
    }

    pub fn print_graph_as_tree(&self) {
        let mut visited = HashSet::new();
        for (from, _) in &self.graph {
            self.print_graph_as_tree_recursive(from, 0, &mut visited);
        }
    }

    fn print_graph_as_tree_recursive(&self, from: &str, depth: usize, visited: &mut HashSet<String>) {
        if visited.contains(from) {
            return;
        }

        visited.insert(from.to_string());

        if let Some(deps) = self.get_dependencies(from) {
            for dep in deps {
                println!("{}{}", "  ".repeat(depth), dep);
                self.print_graph_as_tree_recursive(dep, depth + 1, visited);
            }
        }
    }
}


mod test {
    use super::*;

    #[test]
    fn test_compute_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a".to_string(), "b".to_string());
        graph.add_dependency("a".to_string(), "c".to_string());
        graph.add_dependency("b".to_string(), "c".to_string());
        graph.add_dependency("c".to_string(), "d".to_string());

        let dependencies = graph.compute_dependencies("a");
        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.contains("b"));
        assert!(dependencies.contains("c"));
        assert!(dependencies.contains("d"));

       graph.print_graph();
    }
}