use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    text::{FontAtlasSets, TextPipeline, TextSettings, YAxisOrientation},
    window::WindowRef,
};

use crate::layer_graph::LayerGraph;

#[derive(Component)]
struct ScheduleGraphWindow;

pub fn setup(
    mut commands: Commands,
    layer_graph: Res<LayerGraph>,
    mut text_pipeline: ResMut<TextPipeline>,
    fonts: Res<Assets<Font>>,
    mut font_atlas_sets: ResMut<FontAtlasSets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut textures: ResMut<Assets<Image>>,
    text_settings: Res<TextSettings>,
) {
    let window_comp = Window {
        title: "Schedule Graph".to_string(),
        ..default()
    };
    let scale_factor = window_comp.resolution.scale_factor();
    // Window
    let window = WindowRef::Entity(commands.spawn((window_comp, ScheduleGraphWindow)).id());

    // Camera
    commands.spawn(Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Window(window),
            ..default()
        },
        ..default()
    });

    // commands
    //     .spawn(NodeBundle {
    //         style: Style {
    //             display: Display::Flex,
    //             width: Val::Percent(100.0),
    //             flex_direction: FlexDirection::Row,
    //             align_items: AlignItems::Center,
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .with_children(|builder| {
    //         for layer in &layer_graph.layers {
    //             builder
    //                 .spawn(NodeBundle {
    //                     style: Style {
    //                         flex_direction: FlexDirection::Column,
    //                         margin: UiRect::right(Val::Px(5.)),
    //                         ..default()
    //                     },
    //                     ..default()
    //                 })
    //                 .with_children(|builder| {
    //                     for node in layer {
    //                         builder
    //                             .spawn(NodeBundle {
    //                                 style: Style {
    //                                     margin: UiRect::top(Val::Px(5.)),
    //                                     padding: UiRect::axes(Val::Px(5.), Val::Px(1.)),
    //                                     ..default()
    //                                 },
    //                                 background_color: Color::rgb(0.65, 0.65, 0.65).into(),
    //                                 ..default()
    //                             })
    //                             .with_children(|builder| {
    //                                 builder.spawn(TextBundle::from_section(
    //                                     layer_graph.node_name(node),
    //                                     TextStyle {
    //                                         font_size: 24.0,
    //                                         color: Color::WHITE,
    //                                         ..default()
    //                                     },
    //                                 ));
    //                             });
    //                     }
    //                 });
    //         }
    //     });

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            let mut x_placement = 10.0;
            for layer in &layer_graph.layers {
                let mut layer_width: f32 = 0.0;
                for (y, node) in layer.iter().enumerate() {
                    let name = layer_graph.node_name(node);

                    // So that we can get its size
                    let text_width = match text_pipeline.queue_text(
                        &fonts,
                        &[TextSection::new(
                            name.clone(),
                            TextStyle {
                                font_size: 30.0,
                                ..default()
                            },
                        )],
                        scale_factor,
                        JustifyText::Left,
                        bevy::text::BreakLineOn::NoWrap,
                        Vec2::new(f32::INFINITY, f32::INFINITY),
                        &mut font_atlas_sets,
                        &mut texture_atlas_layouts,
                        &mut textures,
                        text_settings.as_ref(),
                        YAxisOrientation::BottomToTop,
                    ) {
                        Err(e) => panic!("{:?}", e),
                        Ok(info) => info.logical_size.x,
                    };

                    layer_width = layer_width.max(text_width);

                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                margin: UiRect::top(Val::Px(5.)),
                                padding: UiRect::axes(Val::Px(5.), Val::Px(1.)),
                                left: Val::Px(x_placement),
                                top: Val::Px(y as f32 * 35.0 + 5.0),
                                ..default()
                            },
                            background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                            ..default()
                        })
                        .with_children(|builder| {
                            builder.spawn(
                                TextBundle::from_section(
                                    layer_graph.node_name(node),
                                    TextStyle {
                                        font_size: 24.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                )
                                .with_no_wrap(),
                            );
                        });
                }

                x_placement += layer_width / 1.2 + 10.0;
            }
        });
}
