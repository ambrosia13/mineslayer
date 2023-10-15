mod map;

use bevy::{prelude::*, window::WindowResolution};
use map::Map;

pub const WINDOW_SIZE: u32 = 800;
pub const HALF_WINDOW_SIZE: f32 = 400.0;

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
        .add_systems(Startup, (spawn_camera, spawn_map))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

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
    let tile_size = 2.0 * HALF_WINDOW_SIZE / map::MAP_SIZE as f32;

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
                x as f32 * tile_size - HALF_WINDOW_SIZE + 0.5 * tile_size,
                y as f32 * tile_size - HALF_WINDOW_SIZE + 0.5 * tile_size,
                0.0,
            );

            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(tile_size)),
                    ..Default::default()
                },
                transform: current_pos_transform,
                ..Default::default()
            });

            // Spawn text

            if let map::Tile::Neighbor(count) = tile {
                info!("Spawning text for mine neighbor");

                current_pos_transform.translation.z += 0.1;

                commands.spawn(Text2dBundle {
                    text: Text::from_section(format!("{count}"), text_style.clone())
                        .with_alignment(text_alignment),
                    transform: current_pos_transform,
                    ..Default::default()
                });
            }
        }
    }

    commands.spawn(map);
}
