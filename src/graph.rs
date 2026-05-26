use petgraph::graph::UnGraph;
use rand::Rng;

pub fn build_random_graph(n: usize, edge_prob: f32, rng: &mut impl Rng) -> UnGraph<(), f32> {
    let mut graph = UnGraph::new_undirected();
    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(())).collect();
    for i in 0..n {
        for j in (i + 1)..n {
            if rng.random::<f32>() < edge_prob {
                let weight = rng.random::<f32>() * 0.9 + 0.1;
                graph.add_edge(nodes[i], nodes[j], weight);
            }
        }
    }
    graph
}

pub fn build_chain(n: usize) -> UnGraph<(), f32> {
    let mut graph = UnGraph::new_undirected();
    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(())).collect();
    for i in 0..n.saturating_sub(1) {
        graph.add_edge(nodes[i], nodes[i + 1], 1.0);
    }
    graph
}

pub fn build_ring(n: usize) -> UnGraph<(), f32> {
    let mut graph = UnGraph::new_undirected();
    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(())).collect();
    for i in 0..n {
        graph.add_edge(nodes[i], nodes[(i + 1) % n], 1.0);
    }
    graph
}

pub fn build_grid(rows: usize, cols: usize) -> UnGraph<(), f32> {
    let mut graph = UnGraph::new_undirected();
    let nodes: Vec<_> = (0..rows * cols).map(|_| graph.add_node(())).collect();
    for r in 0..rows {
        for c in 0..cols {
            let idx = r * cols + c;
            if c + 1 < cols {
                graph.add_edge(nodes[idx], nodes[idx + 1], 1.0);
            }
            if r + 1 < rows {
                graph.add_edge(nodes[idx], nodes[idx + cols], 1.0);
            }
        }
    }
    graph
}

pub fn build_small_world(n: usize, k: usize, beta: f32, rng: &mut impl Rng) -> UnGraph<(), f32> {
    let mut graph = UnGraph::new_undirected();
    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(())).collect();

    for i in 0..n {
        for d in 1..=(k / 2) {
            let j = (i + d) % n;
            if graph.find_edge(nodes[i], nodes[j]).is_none() {
                graph.add_edge(nodes[i], nodes[j], 1.0);
            }
        }
    }

    let all_edges: Vec<_> = graph.edge_indices().collect();
    for edge_idx in all_edges {
        if rng.random::<f32>() < beta {
            let (src, _dst) = graph.edge_endpoints(edge_idx).unwrap();
            let new_target = loop {
                let t = nodes[rng.random_range(0..n)];
                if t != src && graph.find_edge(src, t).is_none() {
                    break t;
                }
            };
            graph.remove_edge(edge_idx);
            graph.add_edge(src, new_target, 1.0);
        }
    }

    graph
}

pub fn build_scale_free(n: usize, m: usize, rng: &mut impl Rng) -> UnGraph<(), f32> {
    let mut graph = UnGraph::new_undirected();
    let m = m.max(1);

    let seed_count = (m + 1).min(n);
    let mut nodes: Vec<_> = (0..seed_count).map(|_| graph.add_node(())).collect();

    for i in 0..seed_count {
        for j in (i + 1)..seed_count {
            graph.add_edge(nodes[i], nodes[j], 1.0);
        }
    }

    let mut degrees: Vec<usize> = vec![seed_count - 1; seed_count];

    for _ in seed_count..n {
        let new_node = graph.add_node(());
        let total_deg: usize = degrees.iter().sum();
        let mut connected = 0;
        let mut attempts = 0;
        while connected < m && attempts < n * 10 {
            attempts += 1;
            let r = rng.random_range(0..total_deg.max(1));
            let mut cumsum = 0;
            for (idx, &deg) in degrees.iter().enumerate() {
                cumsum += deg;
                if r < cumsum {
                    let target = nodes[idx];
                    if graph.find_edge(new_node, target).is_none() {
                        graph.add_edge(new_node, target, 1.0);
                        degrees[idx] += 1;
                        connected += 1;
                    }
                    break;
                }
            }
        }
        degrees.push(connected);
        nodes.push(new_node);
    }

    graph
}
