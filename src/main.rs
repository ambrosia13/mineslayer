mod map;

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};
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
        .add_systems(
            Update,
            (
                redraw_map,
                detect_tile_updates,
                (handle_tile_updates, propagate_visibility).chain(),
                reset_map,
            ),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource, Default)]
pub struct MapNeedsRedraw(bool);

#[derive(Component)]
pub struct TileReference(usize, usize);

pub enum TileUpdateType {
    FlagPlaced,
    Revealed,
}

#[derive(Component)]
pub struct TileUpdate((usize, usize), TileUpdateType);

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning map");

    let font = asset_server.load("fonts/Inter-Regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 30.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Center;

    let map = Map::new(map::MINE_COUNT);

    for x in 0..map::MAP_SIZE {
        for y in 0..map::MAP_SIZE {
            let tile_display = map.get_at((x, y));
            let color = tile_display.get_color();

            let TileDisplay(tile_is_visible, tile) = tile_display;

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
            info!("Spawning text for mine neighbor");

            current_pos_transform.translation.z = TEXT_Z;

            let text = match tile {
                map::Tile::Neighbor(count) => {
                    if tile_is_visible {
                        format!("{count}")
                    } else {
                        String::new()
                    }
                }
                _ => String::new(),
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

fn detect_tile_updates(
    mut commands: Commands,
    mouse_buttons: Res<Input<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let left_pressed = mouse_buttons.just_pressed(MouseButton::Left);
    let right_pressed = mouse_buttons.just_pressed(MouseButton::Right);

    if left_pressed || right_pressed {
        let update_type = if left_pressed {
            TileUpdateType::Revealed
        } else {
            TileUpdateType::FlagPlaced
        };

        let window = window_query.get_single().unwrap();

        if let Some(cursor_position) = window.cursor_position() {
            let cursor_position =
                Vec2::new(cursor_position.x, WINDOW_SIZE as f32 - cursor_position.y);

            let tile_index = (cursor_position / TILE_SIZE).as_ivec2();
            let tile_index = (tile_index.x as usize, tile_index.y as usize);

            commands.spawn(TileUpdate(tile_index, update_type));
        }
    }
}

fn handle_tile_updates(
    mut commands: Commands,
    tile_update_query: Query<(Entity, &TileUpdate)>,
    mut needs_redraw: ResMut<MapNeedsRedraw>,
    mut map_query: Query<&mut Map>,
) {
    let mut map = map_query.get_single_mut().unwrap();
    let mut update_occurred = false;

    for (entity, TileUpdate(tile_index, update_type)) in tile_update_query.iter() {
        match *update_type {
            TileUpdateType::FlagPlaced => todo!(),
            TileUpdateType::Revealed => map.set_visibility_at(*tile_index, true),
        }

        commands.entity(entity).despawn();

        update_occurred = true;
    }

    needs_redraw.0 = update_occurred;
}

fn propagate_visibility(mut map_query: Query<&mut Map>, mut needs_redraw: ResMut<MapNeedsRedraw>) {
    let mut map = map_query.get_single_mut().unwrap();

    let visibility_changed = map.propagate_visibility();
    if visibility_changed {
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

        let text_string = match tile {
            map::Tile::Neighbor(count) => {
                if tile_is_visible {
                    format!("{count}")
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        };

        text.sections = vec![TextSection::new(text_string, text_style.clone())];
    }

    // We just redrew, don't need to anymore
    needs_redraw.0 = false;
}

fn reset_map(
    mut commands: Commands,
    mut needs_redraw: ResMut<MapNeedsRedraw>,
    map_query: Query<Entity, With<Map>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let map_entity = map_query.get_single().unwrap();

        commands.entity(map_entity).despawn();
        commands.spawn(Map::new(map::MINE_COUNT));

        needs_redraw.0 = true;
    }
}
