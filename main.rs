use std::collections::HashMap;
use petgraph::graph::{Graph, NodeIndex};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let file_path = "Final_Project_Dataset.csv";
    let tour_capacity = calculate_tour_capacity(file_path);
    let mut graph = Graph::<&str, u32>::new();

    // Representing each tour variety as a Node, add nodes to the graph
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();
    for tour_id in tour_capacity.keys() {
        let node_index = graph.add_node(tour_id);
        node_indices.insert(tour_id.to_string(), node_index);
    }

    // Our sink node s and our source node t are not included in the dataset, so they need to be generated.
    let source = graph.add_node("source");
    let sink = graph.add_node("sink");

    // Since they're independent, need to connect to the rest of the dataset
    for (tour_id, capacity) in tour_capacity.iter() {
        let tour_node = node_indices.get(tour_id).expect("Tour node not found in node_indices");
        graph.add_edge(source, *tour_node, *capacity);
        graph.add_edge(*tour_node, sink, 56); // Each tour has a capacity of 56 riders.
    }

    // Find max flow, min cut
    let (max_flow, cut) = ford_fulkerson(&graph, source, sink);
    println!("Maximum flow: {}", max_flow);
    println!("Maximun possible flow: {}", 56*835); //This is largely for sake of understanding. If each edge weight is the capacity, this is the result. Aka if every tour sold out. 883 is our node count.
    println!("Minimum cut: {:?}", cut);
    // Disclaimer: you will find that the first output is "Invalid Rider Count" on line 1. This is because line 1 of the dataset is the title, "Final_Project_Dataset"
}


// "tour capacity" is not exactly "calculated" here. The number of riders on each row is grouped within the node to calculate the amount of riders that are signed up. 
// "capacity" remains constant with an edge threshold of 56 people This comes up earlier in main.
fn calculate_tour_capacity(file_path: &str) -> HashMap<String, u32> {
    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut tour_capacity: HashMap<String, u32> = HashMap::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.expect("Failed to read line");
        let columns: Vec<_> = line.split(',').collect();
        let tour_id = columns[38].to_owned();
        let rider_count = columns[26].parse::<u32>();
        match rider_count {
            Ok(count) => {
                let capacity = tour_capacity.entry(tour_id).or_insert(0);
                *capacity += count;
            }
            Err(e) => {
                println!("Invalid rider count on line {}: {}", line_num + 1, e);

            }
        }
    }

    tour_capacity
}



// Ford fulkerson implementation
fn ford_fulkerson(graph: &Graph<&str, u32>, source: NodeIndex, sink: NodeIndex) -> (u32, Vec<NodeIndex>) {
    let mut flow_graph = graph.clone();
    let mut flow = 0;
    let mut path = find_path(&flow_graph, source, sink);

    while let Some(p) = path {
        let mut min_capacity = u32::max_value();

        for i in 0..p.len() - 1 {
            let capacity = *flow_graph.edge_weight(flow_graph.find_edge(p[i], p[i+1]).unwrap()).unwrap();
            if capacity < min_capacity {
                min_capacity = capacity;
            }
        }

        for i in 0..p.len() - 1 {
            let edge = flow_graph.find_edge(p[i], p[i+1]).unwrap();
            let capacity = *flow_graph.edge_weight(edge).unwrap();
            let new_capacity = capacity - min_capacity;

            flow_graph.update_edge(p[i], p[i+1], new_capacity);

            // Add or update reverse edge
            if let Some(reverse_edge) = flow_graph.find_edge(p[i+1], p[i]) {
                let reverse_capacity = *flow_graph.edge_weight(reverse_edge).unwrap();
                let new_reverse_capacity = reverse_capacity + min_capacity;
                flow_graph.update_edge(p[i+1], p[i], new_reverse_capacity);
            } else {
                flow_graph.add_edge(p[i+1], p[i], min_capacity);
            }
        }

        flow += min_capacity;
        path = find_path(&flow_graph, source, sink);
    }

    let cut: Vec<NodeIndex> = flow_graph.node_indices().filter(|&n| dfs(&flow_graph, n, sink)).collect();
    (flow, cut)
}

// Using depth-first search. This is available under a variant of the Ford Fulkerson algorithm knowbn as "Edmonds Karp" but ultimately I chose not to use it. 
fn dfs(graph: &Graph<&str, u32>, current: NodeIndex, target: NodeIndex) -> bool {
    let mut visited = vec![false; graph.node_count()];
    let mut stack = Vec::new();
    stack.push(current);

    while let Some(node) = stack.pop() {
        if !visited[node.index()] {
            visited[node.index()] = true;

            if node == target {
                return true;
            }

            for neighbor in graph.neighbors(node) {
                let edge = graph.find_edge(node, neighbor).unwrap();
                if *graph.edge_weight(edge).unwrap() > 0 {
                    stack.push(neighbor);
                }
            }
        }
    }

    false
}


// Using breadth-first search 
fn find_path(graph: &Graph<&str, u32>, source: NodeIndex, sink: NodeIndex) -> Option<Vec<NodeIndex>> {
    let mut visited = vec![false; graph.node_count()];
    let mut previous = vec![None; graph.node_count()];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(source);
    visited[source.index()] = true;

    while let Some(current) = queue.pop_front() {
        if current == sink {
            let mut path = vec![sink];

            let mut p = current;
            while p != source {
                let index = p.index();
                p = previous[index].unwrap();
                path.push(p);
            }

            path.reverse();
            return Some(path);
        }

        for neighbor in graph.neighbors(current) {
            if !visited[neighbor.index()] && *graph.edge_weight(graph.find_edge(current, neighbor).unwrap()).unwrap() > 0 {
                visited[neighbor.index()] = true;
                previous[neighbor.index()] = Some(current);
                queue.push_back(neighbor);
            }
        }
    }

    None
}