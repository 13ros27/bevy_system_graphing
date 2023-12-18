use bevy::{
    ecs::schedule::{NodeId, ScheduleLabel},
    prelude::*,
    utils::{
        petgraph::{self, graphmap::DiGraphMap},
        HashMap,
    },
};

use crate::{
    graph_ui::setup,
    graph_utils::*,
    layer_graph::{LayerGraph, LayerNode},
    shorten_type::shorten_systems,
};

pub struct ScheduleGraphPlugin;

impl Plugin for ScheduleGraphPlugin {
    fn finish(&self, app: &mut App) {
        let layer_graph = build_schedule_graph(In(PostUpdate), &mut app.world);
        app.insert_resource(layer_graph).add_systems(Startup, setup);
    }
    fn build(&self, _app: &mut App) {}
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct ScheduleDebugGroup;

// TODO: Figure out how to run this as a 'normal' system
fn build_schedule_graph<S: ScheduleLabel + Clone>(
    In(schedule_label): In<S>,
    world: &mut World,
) -> LayerGraph {
    world.resource_scope::<Schedules, _>(|world, mut schedules| {
        let ignored_ambiguities = schedules.ignored_scheduling_ambiguities.clone();

        let schedule = schedules.get_mut(schedule_label).unwrap();
        schedule.graph_mut().initialize(world);
        let _ = schedule.graph_mut().build_schedule(
            world.components(),
            ScheduleDebugGroup.intern(),
            &ignored_ambiguities,
        );

        schedule_graph_layout(schedule)
    })
}

fn schedule_graph_layout(schedule: &Schedule) -> LayerGraph {
    let graph = schedule.graph();
    let mut dependencies = graph.dependency().graph().clone();
    let mut hierarchy = graph.hierarchy().graph().clone();
    let topsort = graph.dependency().cached_topsort().to_vec();

    // Remove all trivial sets
    for node in topsort {
        if node.is_set() {
            let mut systems = hierarchy.neighbors_directed(node, petgraph::Direction::Outgoing);

            // Remove trivial sets (ones with only one system)
            if let Some(system_node) = systems.next() {
                if systems.next().is_none() {
                    let incoming_set_deps: Vec<_> = dependencies
                        .neighbors_directed(node, petgraph::Direction::Incoming)
                        .collect();
                    let outgoing_set_deps: Vec<_> = dependencies
                        .neighbors_directed(node, petgraph::Direction::Outgoing)
                        .collect();

                    for incoming_node in incoming_set_deps {
                        dependencies.add_edge(incoming_node, system_node, ());
                    }

                    for outgoing_node in outgoing_set_deps {
                        dependencies.add_edge(system_node, outgoing_node, ());
                    }

                    dependencies.remove_node(node);
                    hierarchy.remove_node(node);
                }
            } else {
            }
        }
    }

    let topsort = topsort_graph(&dependencies).unwrap();
    // Simplify duplicates dependencies between systems and their sets
    // for &node in &topsort {
    //     if node.is_set() {
    //         let systems: Vec<_> = hierarchy
    //             .neighbors_directed(node, petgraph::Direction::Outgoing)
    //             .collect();

    //         let incoming_set_deps: HashSet<_> = dependencies
    //             .neighbors_directed(node, petgraph::Direction::Incoming)
    //             .collect();
    //         let outgoing_set_deps: HashSet<_> = dependencies
    //             .neighbors_directed(node, petgraph::Direction::Outgoing)
    //             .collect();

    //         // Remove dependencies on systems that also apply to their set
    //         for system in systems {
    //             let incoming_deps: HashSet<_> = dependencies
    //                 .neighbors_directed(system, petgraph::Direction::Incoming)
    //                 .collect();
    //             let outgoing_deps: HashSet<_> = dependencies
    //                 .neighbors_directed(system, petgraph::Direction::Outgoing)
    //                 .collect();

    //             for &shared in incoming_deps.intersection(&incoming_set_deps) {
    //                 dependencies.remove_edge(shared, system);
    //             }
    //             for &shared in outgoing_deps.intersection(&outgoing_set_deps) {
    //                 dependencies.remove_edge(system, shared);
    //             }
    //         }
    //     }
    // }

    let graph_info = check_graph(&dependencies, &topsort);
    let graph_map = graph_info.transitive_reduction; // Is this doing anything?

    // This is just setless systems and sets with added constraints from all their systems so they layer correctly
    let mut layering_graph = DiGraphMap::new(); // Is it more efficient to capacity this (it will overallocate)
    for &node in &topsort {
        // System without a set should be added
        if node.is_set()
            || (node.is_system()
                && hierarchy
                    .neighbors_directed(node, petgraph::Direction::Incoming)
                    .next()
                    .is_none())
        {
            layering_graph.add_node(node);
            for child_node in dependencies.neighbors_directed(node, petgraph::Direction::Outgoing) {
                if child_node.is_set()
                    || (child_node.is_system()
                        && hierarchy
                            .neighbors_directed(child_node, petgraph::Direction::Incoming)
                            .next()
                            .is_none())
                {
                    layering_graph.add_edge(node, child_node, ());
                } else {
                    for set in
                        hierarchy.neighbors_directed(child_node, petgraph::Direction::Incoming)
                    {
                        // Don't create mini-cycles
                        if node != set {
                            layering_graph.add_edge(node, set, ());
                        }
                    }
                }
            }
        }

        if node.is_set() {
            for system in hierarchy.neighbors_directed(node, petgraph::Direction::Outgoing) {
                // TODO: Dedup this
                for child_node in
                    dependencies.neighbors_directed(system, petgraph::Direction::Outgoing)
                {
                    if child_node.is_set()
                        || (child_node.is_system()
                            && hierarchy
                                .neighbors_directed(child_node, petgraph::Direction::Incoming)
                                .next()
                                .is_none())
                    {
                        layering_graph.add_edge(node, child_node, ());
                    } else {
                        for set in
                            hierarchy.neighbors_directed(child_node, petgraph::Direction::Incoming)
                        {
                            // Don't create mini-cycles
                            if node != set {
                                layering_graph.add_edge(node, set, ());
                            }
                        }
                    }
                }
            }
        }
    }
    let layer_topsort = topsort_graph(&layering_graph).unwrap();

    let mut layers: HashMap<NodeId, usize> = HashMap::new();
    for &node in &layer_topsort {
        if node.is_set()
            || (node.is_system()
                && hierarchy
                    .neighbors_directed(node, petgraph::Direction::Incoming)
                    .next()
                    .is_none())
        {
            let incoming_neighbours = layering_graph
                .neighbors_directed(node, petgraph::Direction::Incoming)
                .collect::<Vec<_>>();

            if incoming_neighbours.is_empty() {
                layers.insert(node, 0);
            } else {
                let parent_layer = incoming_neighbours.iter().map(|n| layers[n]).max().unwrap();
                layers.insert(node, parent_layer + 1);
            }
        }
    }

    let mut layers_vec = Vec::new();
    for (node, layer) in layers {
        if layer >= layers_vec.len() {
            layers_vec.resize(layer + 1, Vec::new());
        }
        layers_vec[layer].push(node);
    }

    let node_names = shorten_systems(graph.systems().map(|(n, s, _)| (n, s.name())).collect())
        .into_iter()
        .chain(graph.system_sets().map(|(n, s, _)| (n, format!("{:?}", s))))
        .collect::<HashMap<_, _>>();

    let mut layer_graph = LayerGraph::default();
    for layer in &layers_vec {
        for &node in layer {
            if node.is_system() {
                layer_graph.add_node(node, LayerNode::System(node_names[&node].clone()));
            } else if node.is_set() {
                // TODO: Add a sub layer graph
                layer_graph.add_node(
                    node,
                    LayerNode::Set(node_names[&node].clone(), LayerGraph::default()),
                );
            }

            layer_graph.add_edges(
                node,
                graph_map
                    .neighbors_directed(node, petgraph::Direction::Outgoing)
                    .collect(),
            );
        }
    }
    layer_graph.layers = layers_vec;

    layer_graph
}
