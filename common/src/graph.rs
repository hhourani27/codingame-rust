use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::Error as IoError;
use std::io::Write;
use std::thread::current;

pub struct Graph<N: Eq + Hash + Copy + Display, E: Copy + Display> {
    nodes: Vec<N>,
    edges_out: Vec<Vec<(E, usize)>>,
    edges_in: Vec<Vec<(usize, E)>>,

    node_idx: HashMap<N, usize>,
    node_counter: usize,
}

impl<N: Eq + Hash + Copy + Display, E: Eq + Copy + Display> Graph<N, E> {
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

    pub fn print_dot(&self, file_path: &str) -> Result<(), IoError> {
        let mut out: Vec<String> = Vec::new();

        out.push("digraph G {".to_string());

        for (i, node_from) in self.nodes.iter().enumerate() {
            for (edge, node_to_idx) in self.edges_out[i].iter() {
                out.push(format!(
                    "\"{}\" -> \"{}\" [label=\"{}\"]",
                    &node_from, &self.nodes[*node_to_idx], &edge
                ));
            }
        }

        out.push("}".to_string());

        let mut _file = File::create(file_path)?;
        writeln!(_file, "{}", out.join("\r\n"))?;

        Ok(())
    }

    pub fn bfs_node_to_edge(
        &self,
        start_node: &N,
        target_edge: &E,
    ) -> Result<Option<Vec<(E, N)>>, String> {
        use std::collections::VecDeque;

        let mut frontier: VecDeque<usize> = VecDeque::new();
        let start_node_idx = match self.node_idx.get(&start_node) {
            Some(nid) => *nid,
            None => return Err(format!("Couldn't find starting node {}", start_node)),
        };
        frontier.push_back(start_node_idx);

        let mut came_from: HashMap<usize, (usize, E)> = HashMap::new();

        let mut found_target = false;
        let mut current_node_idx = 0;
        'main: while !frontier.is_empty() {
            current_node_idx = frontier.pop_front().unwrap();
            let edges = &self.edges_out[current_node_idx];
            for (edge, to_node_idx) in edges.iter() {
                if edge == target_edge {
                    found_target = true;
                    break 'main;
                } else if !came_from.contains_key(&to_node_idx) {
                    frontier.push_back(*to_node_idx);
                    came_from.insert(*to_node_idx, (current_node_idx, edge.clone()));
                }
            }
        }

        match found_target {
            false => Ok(None),
            true => {
                let mut path: Vec<(E, N)> = Vec::new();
                while current_node_idx != start_node_idx {
                    let (prev_node_idx, edge) = *came_from.get(&current_node_idx).unwrap();
                    path.push((edge, self.nodes[current_node_idx].clone()));
                    current_node_idx = prev_node_idx;
                }
                path.reverse();
                Ok(Some(path))
            }
        }
    }
}
