use macroquad::{audio, prelude::*};
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::rc::Rc;

pub const CELL_SIZE: f32 = 32.0;
pub const ENEMY_COLOR: Color = RED;
pub const PLAYER_COLOR: Color = GREEN;
pub const BUILDING_COLOR: Color = WHITE;
pub const MISSILE_SIZE: f32 = CELL_SIZE;
pub const PLANE_WIDTH: f32 = CELL_SIZE;
pub const PLANE_HEIGHT: f32 = 16.0;
pub const CANNON_BASE_WIDTH: f32 = CELL_SIZE;
pub const CANNON_BASE_HEIGHT: f32 = CELL_SIZE;
pub const CANNON_BARREL_WIDTH: f32 = CELL_SIZE;
pub const CANNON_BARREL_HEIGHT: f32 = CELL_SIZE;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

#[derive(Deserialize)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub id: u8,
}

#[derive(Deserialize)]
pub struct LevelData {
    pub buildings: Vec<Entity>,
    pub cannons: Vec<Entity>,
    pub planes: Vec<Entity>,
    pub enemy_missiles: Vec<Entity>,
    pub ground: Vec<Entity>,
}

#[derive(PartialEq)]
pub struct Missile {
    pub x: f32,
    pub y: f32,
    pub direction: Vec2,
    pub speed: f32,
    pub trail_length: f32,
    pub should_destroy: bool,
}

impl Missile {
    pub fn new(x: f32, y: f32, direction: Vec2, speed: f32) -> Missile {
        Missile {
            x,
            y,
            direction,
            speed,
            trail_length: 0.0,
            should_destroy: false,
        }
    }
}

#[derive(PartialEq)]
pub struct Building {
    pub x: f32,
    pub y: f32,
    pub size: Vec2,
    pub texture: Rc<Texture2D>,
    pub should_destroy: bool,
}

#[derive(PartialEq)]
pub struct Plane {
    pub x: f32,
    pub y: f32,
    pub direction: Vec2,
    pub speed: f32,
    pub size: Vec2,
    pub should_destroy: bool,
}

#[derive(Clone, Copy)]
pub struct Cannon {
    pub x: f32,
    pub y: f32,
    pub target: Vec2,
    pub ammo: u32,
}

pub struct Crosshair {
    pub x: f32,
    pub y: f32,
    pub should_destroy: bool,
    pub missile_index: usize,
}

pub struct Game {
    pub buildings: Vec<Building>,
    pub planes: Vec<Plane>,
    pub enemy_missiles_spawnpoints: Vec<Entity>,
    pub ground_entities: Vec<Entity>,
    pub enemy_missiles: Vec<Missile>,
    pub player_missiles: Vec<Missile>,
    pub cannons: Vec<Cannon>,
    pub plane_texture: Rc<Texture2D>,
    pub building_textures: Vec<Rc<Texture2D>>,
    pub missile_texture: Rc<Texture2D>,
    pub cannon_base_texture: Rc<Texture2D>,
    pub cannon_barrel_texture: Rc<Texture2D>,
    pub ground_texture: Rc<Texture2D>,
    pub missile_fire_sound: Rc<audio::Sound>,
    pub explosion_sound: Rc<audio::Sound>,
    pub enemy_missile_sound: Rc<audio::Sound>,
    pub crosshairs: Vec<Crosshair>,
    pub camera: Camera2D,
    pub game_over: bool,
    pub score: i32,
}
