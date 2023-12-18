#![allow(dead_code)] // While prototyping

mod graph_ui;
mod graph_utils;
mod layer_graph;
mod schedule_graph;
mod shorten_type;

use bevy::prelude::*;

use crate::schedule_graph::ScheduleGraphPlugin;

#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
struct TestSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
struct TestSet2;

fn test1() {}
fn test2() {}
fn test3() {}
fn test4() {}

struct ShortenTest1;
impl ShortenTest1 {
    fn test() {}
}

struct ShortenTest2;
impl ShortenTest2 {
    fn test() {}
}

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, ScheduleGraphPlugin))
        .add_systems(
            Update,
            (
                (
                    test1.before(test2).before(TestSet),
                    test3.after(test1).in_set(TestSet),
                    test2.after(test3).in_set(TestSet),
                )
                    .chain(),
                test4.after(test3),
                // ShortenTest1::test,
                // ShortenTest2::test,
            ),
        );
    //     .add_systems(
    //         Update,
    //         (
    //             test1.before(TestSet2),
    //             test2.in_set(TestSet),
    //             test3.in_set(TestSet2).in_set(TestSet),
    //         ),
    //     );
    app.run();
}
