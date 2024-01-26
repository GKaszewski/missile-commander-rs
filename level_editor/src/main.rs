use macroquad::prelude::*;
use serde::{ Serialize, Deserialize };
use serde_json;

const GRID_CELL_SIZE: f32 = 32.0;
const BUIDLING_SIZE: f32 = GRID_CELL_SIZE * 2.0;
const CANNON_SIZE: f32 = GRID_CELL_SIZE;
const PLANE_SIZE: f32 = GRID_CELL_SIZE;
const PLACEMENT_TYPES_NUM: usize = 4;

#[derive(Serialize, Deserialize)]
struct Entity {
    x: f32,
    y: f32,
}

/*
    Enemy missiles is a vector of enemy missiles spawnpoints.
*/

#[derive(Serialize, Deserialize)]
struct Level {
    buildings: Vec<Entity>,
    cannons: Vec<Entity>,
    planes: Vec<Entity>,
    enemy_missiles: Vec<Entity>,
}

enum Placement {
    Building,
    Cannon,
    Plane,
    EnemyMissile,
}

struct EditorState {
    current_placement: Placement,
    current_placement_index: usize,
}

fn draw_building(x: f32, y: f32, texture: &Texture2D, color: Color) {
    draw_texture_ex(texture, x, y, color, DrawTextureParams {
        dest_size: Some(vec2(BUIDLING_SIZE, BUIDLING_SIZE)),
        ..Default::default()
    });
}

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Missile Editor - Level Editor"),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

fn draw_editor_grid() {
    let screen_width = screen_width();
    let screen_height = screen_height();

    let grid_width = (screen_width / GRID_CELL_SIZE).ceil() as u16;
    let grid_height = (screen_height / GRID_CELL_SIZE).ceil() as u16;

    for x in 0..grid_width {
        draw_line(
            (x as f32) * GRID_CELL_SIZE,
            0.0,
            (x as f32) * GRID_CELL_SIZE,
            screen_height,
            1.0,
            DARKGRAY
        );
    }

    for y in 0..grid_height {
        draw_line(
            0.0,
            (y as f32) * GRID_CELL_SIZE,
            screen_width,
            (y as f32) * GRID_CELL_SIZE,
            1.0,
            DARKGRAY
        );
    }
}

fn draw_pointer(cam: &Camera2D, editor_state: &EditorState, building_texture: &Texture2D) {
    let mouse_pos = mouse_position();
    let mouse_pos = vec2(mouse_pos.0, mouse_pos.1);
    let mouse_pos = cam.screen_to_world(mouse_pos);

    let x = (mouse_pos.x / GRID_CELL_SIZE).floor() * GRID_CELL_SIZE;
    let y = (mouse_pos.y / GRID_CELL_SIZE).floor() * GRID_CELL_SIZE;

    match editor_state.current_placement {
        Placement::Building => {
            draw_building(x, y, &building_texture, Color::from_rgba(255, 255, 255, 100));
        }
        Placement::Cannon => {
            draw_cannon(x, y);
        }
        Placement::Plane => {
            draw_plane(x, y);
        }
        Placement::EnemyMissile => {
            draw_enemy_missile(x, y);
        }
    }
}

fn draw_cannon(x: f32, y: f32) {
    draw_rectangle(x, y, CANNON_SIZE, CANNON_SIZE, GREEN);
    draw_text("C", x + CANNON_SIZE / 2.0, y + CANNON_SIZE / 2.0, 16.0, WHITE);
}

fn draw_plane(x: f32, y: f32) {
    draw_rectangle(x, y, PLANE_SIZE, PLANE_SIZE, BLUE);
    draw_text("P", x + PLANE_SIZE / 2.0, y + PLANE_SIZE / 2.0, 16.0, WHITE);
}

fn draw_enemy_missile(x: f32, y: f32) {
    draw_rectangle(x, y, GRID_CELL_SIZE, GRID_CELL_SIZE, RED);
    draw_text("M", x + GRID_CELL_SIZE / 2.0, y + GRID_CELL_SIZE / 2.0, 16.0, WHITE);
}

fn draw_level(level: &Level, building_texture: &Texture2D) {
    for building in &level.buildings {
        draw_building(building.x, building.y, building_texture, WHITE);
    }

    for cannon in &level.cannons {
        draw_cannon(cannon.x, cannon.y);
    }

    for plane in &level.planes {
        draw_plane(plane.x, plane.y);
    }

    for enemy_missile in &level.enemy_missiles {
        draw_enemy_missile(enemy_missile.x, enemy_missile.y);
    }
}

fn draw_background(texture: &Texture2D) {
    draw_texture_ex(texture, 0.0, 0.0, WHITE, DrawTextureParams {
        dest_size: Some(vec2(screen_width(), screen_height())),
        ..Default::default()
    });
}

fn handle_placement_on_mouse_wheel(editor_state: &mut EditorState) {
    let mouse_wheel = mouse_wheel().1;
    let treshold = 0.1;

    if mouse_wheel > treshold {
        editor_state.current_placement_index += 1;
        editor_state.current_placement_index %= PLACEMENT_TYPES_NUM;
        set_placement_by_index(editor_state);
    } else if mouse_wheel < -treshold {
        if editor_state.current_placement_index == 0 {
            editor_state.current_placement_index = PLACEMENT_TYPES_NUM - 1;
        } else {
            editor_state.current_placement_index -= 1;
        }
        editor_state.current_placement_index %= PLACEMENT_TYPES_NUM;
        set_placement_by_index(editor_state);
    }
}

fn set_placement_by_index(editor_state: &mut EditorState) {
    match editor_state.current_placement_index {
        0 => {
            editor_state.current_placement = Placement::Building;
        }
        1 => {
            editor_state.current_placement = Placement::Cannon;
        }
        2 => {
            editor_state.current_placement = Placement::Plane;
        }
        3 => {
            editor_state.current_placement = Placement::EnemyMissile;
        }
        _ => {}
    }
}

fn get_entity_xy_from_mouse(cam: &Camera2D) -> (f32, f32) {
    let mouse_pos = mouse_position();
    let mouse_pos = vec2(mouse_pos.0, mouse_pos.1);
    let mouse_pos = cam.screen_to_world(mouse_pos);

    let x = (mouse_pos.x / GRID_CELL_SIZE).floor() * GRID_CELL_SIZE;
    let y = (mouse_pos.y / GRID_CELL_SIZE).floor() * GRID_CELL_SIZE;

    (x, y)
}

fn place_building(cam: &Camera2D, level: &mut Level) {
    let (x, y) = get_entity_xy_from_mouse(cam);
    level.buildings.push(Entity { x, y });
}

fn place_cannon(cam: &Camera2D, level: &mut Level) {
    let (x, y) = get_entity_xy_from_mouse(cam);
    level.cannons.push(Entity { x, y });
}

fn place_plane(cam: &Camera2D, level: &mut Level) {
    let (x, y) = get_entity_xy_from_mouse(cam);
    level.planes.push(Entity { x, y });
}

fn place_enemy_missile(cam: &Camera2D, level: &mut Level) {
    let (x, y) = get_entity_xy_from_mouse(cam);
    level.enemy_missiles.push(Entity { x, y });
}

fn place_entity(cam: &Camera2D, level: &mut Level, editor_state: &EditorState) {
    match editor_state.current_placement {
        Placement::Building => {
            place_building(cam, level);
        }
        Placement::Cannon => {
            place_cannon(cam, level);
        }
        Placement::Plane => {
            place_plane(cam, level);
        }
        Placement::EnemyMissile => {
            place_enemy_missile(cam, level);
        }
    }
}

fn get_entity_from_cell<'a>(
    cam: &'a Camera2D,
    entities: &'a Vec<Entity>,
    x: f32,
    y: f32
) -> Option<&'a Entity> {
    let (x, y) = get_entity_xy_from_mouse(cam);
    entities
        .iter()
        .find(|entity| { entity.x == x && entity.y == y })
        .map(|entity| entity)
}

fn remove_entity_from_cell(cam: &Camera2D, entities: &mut Vec<Entity>) {
    let (x, y) = get_entity_xy_from_mouse(cam);
    entities.retain(|entity| { entity.x != x || entity.y != y });
}

fn remove_entity(cam: &Camera2D, level: &mut Level, editor_state: &EditorState) {
    match editor_state.current_placement {
        Placement::Building => {
            remove_entity_from_cell(cam, &mut level.buildings);
        }
        Placement::Cannon => {
            remove_entity_from_cell(cam, &mut level.cannons);
        }
        Placement::Plane => {
            remove_entity_from_cell(cam, &mut level.planes);
        }
        Placement::EnemyMissile => {
            remove_entity_from_cell(cam, &mut level.enemy_missiles);
        }
    }
}

fn save_level(level: &Level) {
    let level_json = serde_json::to_string(level);
    match level_json {
        Ok(level_json) => {
            match std::fs::write("level.json", level_json) {
                Ok(_) => {
                    draw_text(
                        "Level saved!",
                        screen_width() / 2.0,
                        screen_height() / 2.0,
                        72.0,
                        GREEN
                    );
                }
                Err(_) => {
                    draw_text(
                        "Failed to save level!",
                        screen_width() / 2.0,
                        screen_height() / 2.0,
                        72.0,
                        RED
                    );
                }
            }
        }
        Err(_) => {
            draw_text(
                "Failed to save level!",
                screen_width() / 2.0,
                screen_height() / 2.0,
                72.0,
                RED
            );
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let building_texture = load_texture("assets/building.png").await.unwrap();
    let missile_texture = load_texture("assets/missile.png").await.unwrap();
    let background_texture = load_texture("assets/background.png").await.unwrap();
    building_texture.set_filter(FilterMode::Nearest);
    missile_texture.set_filter(FilterMode::Nearest);
    background_texture.set_filter(FilterMode::Nearest);

    let mut level = Level {
        buildings: Vec::new(),
        cannons: Vec::new(),
        planes: Vec::new(),
        enemy_missiles: Vec::new(),
    };

    let mut editor_state = EditorState {
        current_placement: Placement::Building,
        current_placement_index: 0,
    };

    let camera = Camera2D {
        zoom: vec2((1.0 / screen_width()) * 2.0, (1.0 / screen_height()) * 2.0),
        target: vec2(screen_width() / 2.0, screen_height() / 2.0),
        ..Default::default()
    };
    set_camera(&camera);

    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            place_entity(&camera, &mut level, &editor_state);
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            remove_entity(&camera, &mut level, &editor_state);
        }
        handle_placement_on_mouse_wheel(&mut editor_state);

        clear_background(LIGHTGRAY);
        draw_background(&background_texture);
        draw_editor_grid();
        draw_level(&level, &building_texture);
        draw_pointer(&camera, &editor_state, &building_texture);
        if is_key_pressed(KeyCode::S) {
            save_level(&level);
        }

        next_frame().await;
    }
}
