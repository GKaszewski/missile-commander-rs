#![windows_subsystem = "windows"]
use data::{Assets, Game};
use draw::{draw_background, draw_game};
use logic::{handle_resize, load_level, spawn_enemy_missiles, update_game};
use macroquad::{
    audio::{self},
    prelude::*,
};

use std::{
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

mod data;
mod draw;
mod logic;

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Missile commander"),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64,
    );
    let building1_texture = Texture2D::from_file_with_format(
        &Assets::get("building_1.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let building2_texture = Texture2D::from_file_with_format(
        &Assets::get("building_2.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let building3_texture = Texture2D::from_file_with_format(
        &Assets::get("building_3.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let plane_texture = Texture2D::from_file_with_format(
        &Assets::get("plane.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let missile_texture = Texture2D::from_file_with_format(
        &Assets::get("missile.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let background_texture = Texture2D::from_file_with_format(
        &Assets::get("background.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let cannon_base_texture = Texture2D::from_file_with_format(
        &Assets::get("missile_launcher_part_1.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let cannon_barrel_texture = Texture2D::from_file_with_format(
        &Assets::get("missile_launcher_part_2.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let ground_texture = Texture2D::from_file_with_format(
        &Assets::get("ground.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    missile_texture.set_filter(FilterMode::Nearest);
    plane_texture.set_filter(FilterMode::Nearest);
    cannon_base_texture.set_filter(FilterMode::Nearest);
    cannon_barrel_texture.set_filter(FilterMode::Nearest);
    ground_texture.set_filter(FilterMode::Nearest);
    building1_texture.set_filter(FilterMode::Nearest);
    building2_texture.set_filter(FilterMode::Nearest);
    building3_texture.set_filter(FilterMode::Nearest);
    let building_textures = vec![building1_texture, building2_texture, building3_texture];
    let missile_fire_sound =
        audio::load_sound_from_bytes(&Assets::get("missile_fire.ogg").unwrap().data)
            .await
            .unwrap();
    let explosion_sound = audio::load_sound_from_bytes(&Assets::get("explosion.ogg").unwrap().data)
        .await
        .unwrap();
    let enemy_missile_sound =
        audio::load_sound_from_bytes(&Assets::get("enemy_missile.ogg").unwrap().data)
            .await
            .unwrap();

    let mut game = Game {
        buildings: vec![],
        planes: vec![],
        enemy_missiles_spawnpoints: vec![],
        ground_entities: vec![],
        enemy_missiles: vec![],
        player_missiles: vec![],
        cannons: vec![],
        plane_texture: Rc::new(plane_texture),
        building_textures: building_textures
            .iter()
            .map(|t| Rc::new(t.clone()))
            .collect(),
        missile_texture: Rc::new(missile_texture),
        missile_fire_sound: Rc::new(missile_fire_sound),
        cannon_base_texture: Rc::new(cannon_base_texture),
        cannon_barrel_texture: Rc::new(cannon_barrel_texture),
        ground_texture: Rc::new(ground_texture),
        explosion_sound: Rc::new(explosion_sound),
        enemy_missile_sound: Rc::new(enemy_missile_sound),
        crosshairs: vec![],
        score: 0,
        game_over: false,
        camera: Camera2D {
            zoom: vec2((1.0 / screen_width()) * 2.0, (1.0 / screen_height()) * 2.0),
            target: vec2(screen_width() / 2.0, screen_height() / 2.0),
            ..Default::default()
        },
    };

    load_level(&mut game);
    spawn_enemy_missiles(&mut game);

    set_camera(&game.camera);

    loop {
        let last_screen_width = screen_width();
        let last_screen_height = screen_height();
        handle_resize(last_screen_width, last_screen_height, &mut game);
        clear_background(BLACK);
        draw_background(&background_texture);
        update_game(&mut game);
        draw_game(&game);
        next_frame().await;
    }
}
