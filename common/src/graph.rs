use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::Error;
use std::io::Write;

pub struct Graph<N: Eq + Hash + Copy + Display, E: Copy + Display> {
    nodes: Vec<N>,
    edges_out: Vec<Vec<(E, usize)>>,
    edges_in: Vec<Vec<(usize, E)>>,

    node_idx: HashMap<N, usize>,
    node_counter: usize,
}

impl<N: Eq + Hash + Copy + Display, E: Copy + Display> Graph<N, E> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges_out: Vec::new(),
            edges_in: Vec::new(),
            node_idx: HashMap::new(),
            node_counter: 0,
        }
    }

    pub fn add_node(&mut self, node: N) -> usize {
        self.nodes.push(node);
        self.node_idx.insert(node, self.node_counter);
        self.edges_out.push(Vec::new());
        self.edges_in.push(Vec::new());

        self.node_counter += 1;
        self.node_counter - 1
    }

    pub fn add_edge(&mut self, from: N, to: N, edge: E) {
        let from_idx = *self.node_idx.get(&from).unwrap();
        let to_idx = *self.node_idx.get(&to).unwrap();

        self.edges_out[from_idx].push((edge, to_idx));
        self.edges_in[to_idx].push((from_idx, edge));
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        let mut edge_count = 0;
        for n in 0..self.nodes.len() {
            edge_count += self.edges_out[n].len();
        }

        edge_count
    }

    pub fn print_dot(&self, file_path: &str) -> Result<(), Error> {
        let mut out: Vec<String> = Vec::new();

        out.push("digraph G {".to_string());

        for (i, node_from) in self.nodes.iter().enumerate() {
            for (edge, node_to) in self.edges_out[i].iter() {
                out.push(format!(
                    "\"{}\" -> \"{}\" [label=\"{}\"]",
                    &node_from, &node_to, &edge
                ));
            }
        }

        out.push("}".to_string());

        let mut _file = File::create(file_path)?;
        writeln!(_file, "{}", out.join("\r\n"))?;

        Ok(())
    }
}
