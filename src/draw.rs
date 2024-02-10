use crate::data::{
    Building, Cannon, Crosshair, Entity, Game, Missile, Plane, BUILDING_COLOR,
    CANNON_BARREL_HEIGHT, CANNON_BARREL_WIDTH, CANNON_BASE_HEIGHT, CANNON_BASE_WIDTH, ENEMY_COLOR,
    MISSILE_SIZE, PLAYER_COLOR,
};
use macroquad::prelude::*;

pub fn draw_missile(
    x: f32,
    y: f32,
    size: f32,
    direction: Vec2,
    color: Color,
    missile_texture: &Texture2D,
) {
    let rotation = direction.y.atan2(direction.x);
    draw_texture_ex(
        missile_texture,
        x,
        y,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(size, size / 2.0)),
            rotation,
            ..Default::default()
        },
    );
}

pub fn draw_trail(x: f32, y: f32, length: f32, direction: Vec2) {
    draw_line(
        x,
        y,
        x + direction.x * length,
        y + direction.y * length,
        1.0,
        WHITE,
    );
}

pub fn draw_x_crosshair(x: f32, y: f32, size: f32, color: Color) {
    draw_line(x - size, y - size, x + size, y + size, 1.0, color);
    draw_line(x + size, y - size, x - size, y + size, 1.0, color);
}

pub fn draw_cannon(
    x: f32,
    y: f32,
    target: Vec2,
    ammo: u32,
    base_texture: &Texture2D,
    barrel_texture: &Texture2D,
) {
    let direction = target - vec2(x, y);
    let rotation = (direction.y.atan2(direction.x).to_degrees() + 45.0).to_radians();
    draw_texture_ex(
        base_texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(CANNON_BASE_WIDTH, CANNON_BASE_HEIGHT)),
            ..Default::default()
        },
    );
    draw_texture_ex(
        barrel_texture,
        x + 8.0,
        y - CANNON_BARREL_HEIGHT / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(CANNON_BARREL_WIDTH, CANNON_BARREL_HEIGHT)),
            rotation,
            ..Default::default()
        },
    );
    let mut ammo_text = format!("{}", ammo);
    if ammo == 0 {
        ammo_text = "OUT".to_string();
    }
    draw_text(&ammo_text, x - 10.0, y + 10.0, 20.0, WHITE);
}

pub fn draw_building(x: f32, y: f32, size: Vec2, color: Color, building_texture: &Texture2D) {
    draw_texture_ex(
        building_texture,
        x,
        y,
        color,
        DrawTextureParams {
            dest_size: Some(size),
            ..Default::default()
        },
    );
}

pub fn draw_ground(x: f32, y: f32, size: f32, color: Color, ground_texture: &Texture2D) {
    draw_texture_ex(
        ground_texture,
        x,
        y,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );
}

pub fn draw_ground_entities(ground_entities: &Vec<Entity>, ground_texture: &Texture2D) {
    for ground_entity in ground_entities {
        draw_ground(
            ground_entity.x,
            ground_entity.y,
            50.0,
            WHITE,
            ground_texture,
        );
    }
}

pub fn draw_buildings(buildings: &Vec<Building>) {
    for building in buildings {
        draw_building(
            building.x,
            building.y,
            building.size,
            BUILDING_COLOR,
            &building.texture,
        );
        // draw_aabb(
        //     building.x + building.size / 2.0,
        //     building.y + building.size / 2.0,
        //     building.size,
        //     BLUE
        // );
    }
}

pub fn draw_planes(planes: &Vec<Plane>, plane_texture: &Texture2D) {
    for plane in planes {
        let should_flip = plane.direction.x > 0.0;
        draw_texture_ex(
            plane_texture,
            plane.x,
            plane.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(plane.size),
                flip_x: should_flip,
                ..Default::default()
            },
        );
    }
}

pub fn draw_enemy_missiles(enemy_missiles: &Vec<Missile>, missile_texture: &Texture2D) {
    for missile in enemy_missiles {
        draw_trail(
            missile.x + MISSILE_SIZE / 2.0,
            missile.y,
            missile.trail_length,
            -missile.direction,
        );
        draw_missile(
            missile.x,
            missile.y,
            MISSILE_SIZE,
            missile.direction,
            ENEMY_COLOR,
            missile_texture,
        );
        //draw_aabb(missile.x, missile.y, MISSILE_SIZE, BLUE);
    }
}

pub fn draw_player_missiles(player_missiles: &Vec<Missile>, missile_texture: &Texture2D) {
    for missile in player_missiles {
        draw_trail(
            missile.x + MISSILE_SIZE / 2.0,
            missile.y,
            missile.trail_length,
            -missile.direction,
        );
        draw_missile(
            missile.x,
            missile.y,
            MISSILE_SIZE,
            missile.direction,
            WHITE,
            missile_texture,
        );
        //draw_aabb(missile.x, missile.y, MISSILE_SIZE, BLUE);
    }
}

pub fn draw_cannons(
    cannons: &Vec<Cannon>,
    cannon_base_texture: &Texture2D,
    cannon_barrel_texture: &Texture2D,
) {
    for cannon in cannons {
        draw_cannon(
            cannon.x,
            cannon.y,
            cannon.target,
            cannon.ammo,
            cannon_base_texture,
            cannon_barrel_texture,
        );
    }
}

pub fn draw_crosshairs(crosshairs: &Vec<Crosshair>) {
    for crosshair in crosshairs {
        if crosshair.should_destroy {
            continue;
        }

        draw_x_crosshair(crosshair.x, crosshair.y, 10.0, PLAYER_COLOR);
    }
}

pub fn draw_score(score: i32) {
    let score_text = format!("Score: {}", score);
    draw_text(&score_text, 10.0, 20.0, 30.0, WHITE);
}

pub fn draw_game(game: &Game) {
    draw_ground_entities(&game.ground_entities, &game.ground_texture);
    draw_buildings(&game.buildings);
    draw_planes(&game.planes, &game.plane_texture);
    draw_cannons(
        &game.cannons,
        &game.cannon_base_texture,
        &game.cannon_barrel_texture,
    );
    draw_crosshairs(&game.crosshairs);
    draw_enemy_missiles(&game.enemy_missiles, &game.missile_texture);
    draw_player_missiles(&game.player_missiles, &game.missile_texture);
    draw_score(game.score);
}

#[allow(dead_code)]
pub fn draw_aabb(x: f32, y: f32, size: f32, color: Color) {
    draw_rectangle_lines(x - size / 2.0, y - size / 2.0, size, size, 2.0, color);
}

pub fn draw_background(texture: &Texture2D) {
    // draw texture centered on screen and scale it to screen size (without changing aspect ratio)
    let screen_width = screen_width();
    let screen_height = screen_height();
    let texture_width = texture.width();
    let texture_height = texture.height();
    let scale = screen_width / texture_width;
    let x = screen_width / 2.0 - (texture_width / 2.0) * scale;
    let y = screen_height / 2.0 - (texture_height / 2.0) * scale;
    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(texture_width * scale, texture_height * scale)),
            ..Default::default()
        },
    );
}
