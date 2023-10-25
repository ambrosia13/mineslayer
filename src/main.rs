mod map;

use bevy::{prelude::*, window::WindowResolution};
use map::Map;

use crate::map::TileDisplay;

pub const WINDOW_SIZE: u32 = 800;
pub const HALF_WINDOW_SIZE: f32 = 400.0;
pub const TILE_SIZE: f32 = 2.0 * HALF_WINDOW_SIZE / map::MAP_SIZE as f32;
pub const GRID_SIZE: f32 = 2.0;

pub const TILE_Z: f32 = 0.0;
pub const TEXT_Z: f32 = 1.0;
pub const GRID_Z: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_SIZE as f32, WINDOW_SIZE as f32),
                title: "mineslayer".into(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_resource::<MapNeedsRedraw>()
        .add_systems(Startup, (spawn_camera, spawn_map, spawn_grid))
        .add_systems(Update, (request_redraw, redraw_map))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource, Default)]
pub struct MapNeedsRedraw(bool);

#[derive(Component)]
pub struct TileReference(usize, usize);

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning map");

    let font = asset_server.load("fonts/Inter-Regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 30.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Center;

    let map = Map::new(20);

    for x in 0..map::MAP_SIZE {
        for y in 0..map::MAP_SIZE {
            let tile = map.get_at((x, y));
            let color = tile.get_color();

            let TileDisplay(tile_is_visible, tile) = tile;

            let mut current_pos_transform = Transform::from_xyz(
                x as f32 * TILE_SIZE - HALF_WINDOW_SIZE + 0.5 * TILE_SIZE,
                y as f32 * TILE_SIZE - HALF_WINDOW_SIZE + 0.5 * TILE_SIZE,
                TILE_Z,
            );

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..Default::default()
                    },
                    transform: current_pos_transform,
                    ..Default::default()
                },
                TileReference(x, y),
            ));

            // Spawn text
            if let map::Tile::Neighbor(count) = tile {
                info!("Spawning text for mine neighbor");

                current_pos_transform.translation.z = TEXT_Z;

                let text = if tile_is_visible {
                    format!("{count}")
                } else {
                    String::new()
                };

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(text, text_style.clone())
                            .with_alignment(text_alignment),
                        transform: current_pos_transform,
                        ..Default::default()
                    },
                    TileReference(x, y),
                ));
            }
        }
    }

    commands.spawn(map);
}

fn spawn_grid(mut commands: Commands) {
    for i in 0..(map::MAP_SIZE - 1) {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(GRID_SIZE, HALF_WINDOW_SIZE * 2.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(
                i as f32 * TILE_SIZE - HALF_WINDOW_SIZE + TILE_SIZE,
                0.0,
                GRID_Z,
            ),
            ..Default::default()
        });

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(HALF_WINDOW_SIZE * 2.0, GRID_SIZE)),
                ..Default::default()
            },
            transform: Transform::from_xyz(
                0.0,
                i as f32 * TILE_SIZE - HALF_WINDOW_SIZE + TILE_SIZE,
                GRID_Z,
            ),
            ..Default::default()
        });
    }
}

fn request_redraw(mut needs_redraw: ResMut<MapNeedsRedraw>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        needs_redraw.0 = true;
    }
}

fn redraw_map(
    map: Query<&Map>,
    mut sprite_query: Query<(&mut Sprite, &TileReference)>,
    mut text_query: Query<(&mut Text, &TileReference)>,
    mut needs_redraw: ResMut<MapNeedsRedraw>,
    asset_server: Res<AssetServer>,
) {
    // Don't do anything if we don't need redraw
    if !needs_redraw.0 {
        return;
    }

    info!("Redrawing map");

    let map = map.get_single().unwrap();

    // Update tile color
    for (mut sprite, TileReference(x, y)) in sprite_query.iter_mut() {
        let tile = map.get_at((*x, *y));
        sprite.color = tile.get_color();
    }

    // Update text
    let font = asset_server.load("fonts/Inter-Regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 30.0,
        color: Color::BLACK,
    };

    for (mut text, TileReference(x, y)) in text_query.iter_mut() {
        let TileDisplay(tile_is_visible, tile) = map.get_at((*x, *y));

        if let map::Tile::Neighbor(count) = tile {
            let text_string = if tile_is_visible {
                format!("{count}")
            } else {
                String::new()
            };

            text.sections = vec![TextSection::new(text_string, text_style.clone())];
        }
    }

    // We just redrew, don't need to anymore
    needs_redraw.0 = false;
}
