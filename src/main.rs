mod map;

use bevy::{prelude::*, window::WindowResolution};
use map::Map;

pub const WINDOW_SIZE: u32 = 800;
pub const HALF_WINDOW_SIZE: f32 = 400.0;
pub const TILE_SIZE: f32 = 2.0 * HALF_WINDOW_SIZE / map::MAP_SIZE as f32;

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
        .add_systems(Startup, (spawn_camera, spawn_map))
        .add_systems(Update, redraw_map)
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
            let color = match tile {
                map::Tile::Empty => Color::WHITE,
                map::Tile::Neighbor(count) if count == 1 => Color::YELLOW,
                map::Tile::Neighbor(_) => Color::ORANGE,
                map::Tile::Mine => Color::RED,
            };

            let mut current_pos_transform = Transform::from_xyz(
                x as f32 * TILE_SIZE - HALF_WINDOW_SIZE + 0.5 * TILE_SIZE,
                y as f32 * TILE_SIZE - HALF_WINDOW_SIZE + 0.5 * TILE_SIZE,
                0.0,
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

                current_pos_transform.translation.z += 0.1;

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(format!("{count}"), text_style.clone())
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
        let tile = map.get_at((*x, *y));

        if let map::Tile::Neighbor(count) = tile {
            text.sections = vec![TextSection::new(format!("{count}"), text_style.clone())];
        }
    }

    // We just redrew, don't need to anymore
    needs_redraw.0 = false;
}
