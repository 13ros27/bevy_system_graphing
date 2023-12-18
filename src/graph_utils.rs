use bevy::{
    ecs::schedule::{NodeId, ScheduleBuildError},
    utils::{
        petgraph::{
            algo::TarjanScc,
            graphmap::{DiGraphMap, NodeTrait},
            Direction::{Incoming, Outgoing},
        },
        HashMap, HashSet,
    },
};
use fixedbitset::FixedBitSet;
use std::fmt::Debug;

// Copied from bevy/ecs/schedule/graph_utils because it isn't public
pub(crate) fn check_graph<V>(
    graph: &DiGraphMap<V, ()>,
    topological_order: &[V],
) -> CheckGraphResults<V>
where
    V: NodeTrait + Debug,
{
    if graph.node_count() == 0 {
        return CheckGraphResults::default();
    }

    let n = graph.node_count();

    // build a copy of the graph where the nodes and edges appear in topsorted order
    let mut map = HashMap::with_capacity(n);
    let mut topsorted = DiGraphMap::<V, ()>::new();
    // iterate nodes in topological order
    for (i, &node) in topological_order.iter().enumerate() {
        map.insert(node, i);
        topsorted.add_node(node);
        // insert nodes as successors to their predecessors
        for pred in graph.neighbors_directed(node, Incoming) {
            topsorted.add_edge(pred, node, ());
        }
    }

    let mut reachable = FixedBitSet::with_capacity(n * n);
    let mut connected = HashSet::new();
    let mut disconnected = Vec::new();

    let mut transitive_edges = Vec::new();
    let mut transitive_reduction = DiGraphMap::<V, ()>::new();
    let mut transitive_closure = DiGraphMap::<V, ()>::new();

    let mut visited = FixedBitSet::with_capacity(n);

    // iterate nodes in topological order
    for node in topsorted.nodes() {
        transitive_reduction.add_node(node);
        transitive_closure.add_node(node);
    }

    // iterate nodes in reverse topological order
    for a in topsorted.nodes().rev() {
        let index_a = *map.get(&a).unwrap();
        // iterate their successors in topological order
        for b in topsorted.neighbors_directed(a, Outgoing) {
            let index_b = *map.get(&b).unwrap();
            debug_assert!(index_a < index_b);
            if !visited[index_b] {
                // edge <a, b> is not redundant
                transitive_reduction.add_edge(a, b, ());
                transitive_closure.add_edge(a, b, ());
                reachable.insert(index(index_a, index_b, n));

                let successors = transitive_closure
                    .neighbors_directed(b, Outgoing)
                    .collect::<Vec<_>>();
                for c in successors {
                    let index_c = *map.get(&c).unwrap();
                    debug_assert!(index_b < index_c);
                    if !visited[index_c] {
                        visited.insert(index_c);
                        transitive_closure.add_edge(a, c, ());
                        reachable.insert(index(index_a, index_c, n));
                    }
                }
            } else {
                // edge <a, b> is redundant
                transitive_edges.push((a, b));
            }
        }

        visited.clear();
    }

    // partition pairs of nodes into "connected by path" and "not connected by path"
    for i in 0..(n - 1) {
        // reachable is upper triangular because the nodes were topsorted
        for index in index(i, i + 1, n)..=index(i, n - 1, n) {
            let (a, b) = row_col(index, n);
            let pair = (topological_order[a], topological_order[b]);
            if reachable[index] {
                connected.insert(pair);
            } else {
                disconnected.push(pair);
            }
        }
    }

    // fill diagonal (nodes reach themselves)
    // for i in 0..n {
    //     reachable.set(index(i, i, n), true);
    // }

    CheckGraphResults {
        reachable,
        connected,
        disconnected,
        transitive_edges,
        transitive_reduction,
        transitive_closure,
    }
}

// Hey look more pub(crate)
pub(crate) fn index(row: usize, col: usize, num_cols: usize) -> usize {
    debug_assert!(col < num_cols);
    (row * num_cols) + col
}

pub(crate) fn row_col(index: usize, num_cols: usize) -> (usize, usize) {
    (index / num_cols, index % num_cols)
}

// More copy-paste
/// Stores the results of the graph analysis.
pub(crate) struct CheckGraphResults<V> {
    /// Boolean reachability matrix for the graph.
    pub(crate) reachable: FixedBitSet,
    /// Pairs of nodes that have a path connecting them.
    pub(crate) connected: HashSet<(V, V)>,
    /// Pairs of nodes that don't have a path connecting them.
    pub(crate) disconnected: Vec<(V, V)>,
    /// Edges that are redundant because a longer path exists.
    pub(crate) transitive_edges: Vec<(V, V)>,
    /// Variant of the graph with no transitive edges.
    pub(crate) transitive_reduction: DiGraphMap<V, ()>,
    /// Variant of the graph with all possible transitive edges.
    // TODO: this will very likely be used by "if-needed" ordering
    #[allow(dead_code)]
    pub(crate) transitive_closure: DiGraphMap<V, ()>,
}

impl<V: NodeTrait + Debug> Default for CheckGraphResults<V> {
    fn default() -> Self {
        Self {
            reachable: FixedBitSet::new(),
            connected: HashSet::new(),
            disconnected: Vec::new(),
            transitive_edges: Vec::new(),
            transitive_reduction: DiGraphMap::new(),
            transitive_closure: DiGraphMap::new(),
        }
    }
}

// More privatisem
pub fn topsort_graph(graph: &DiGraphMap<NodeId, ()>) -> Result<Vec<NodeId>, ScheduleBuildError> {
    // Tarjan's SCC algorithm returns elements in *reverse* topological order.
    let mut tarjan_scc = TarjanScc::new();
    let mut top_sorted_nodes = Vec::with_capacity(graph.node_count());
    let mut sccs_with_cycles = Vec::new();

    tarjan_scc.run(graph, |scc| {
        // A strongly-connected component is a group of nodes who can all reach each other
        // through one or more paths. If an SCC contains more than one node, there must be
        // at least one cycle within them.
        if scc.len() > 1 {
            sccs_with_cycles.push(scc.to_vec());
        }
        top_sorted_nodes.extend_from_slice(scc);
    });

    if sccs_with_cycles.is_empty() {
        // reverse to get topological order
        top_sorted_nodes.reverse();
        Ok(top_sorted_nodes)
    } else {
        unreachable!()
    }
}
