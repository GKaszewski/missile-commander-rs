#![windows_subsystem = "windows"]
use macroquad::{ audio::{ self, play_sound_once }, prelude::* };
use std::{ time::{ SystemTime, UNIX_EPOCH }, rc::Rc };
use serde::Deserialize;
use serde_json;
use rust_embed::RustEmbed;

const CELL_SIZE: f32 = 32.0;
const ENEMY_COLOR: Color = RED;
const PLAYER_COLOR: Color = GREEN;
const BUILDING_COLOR: Color = WHITE;
const MISSILE_SIZE: f32 = CELL_SIZE;
const PLANE_WIDTH: f32 = CELL_SIZE;
const PLANE_HEIGHT: f32 = 16.0;
const CANNON_BASE_WIDTH: f32 = CELL_SIZE;
const CANNON_BASE_HEIGHT: f32 = CELL_SIZE;
const CANNON_BARREL_WIDTH: f32 = CELL_SIZE;
const CANNON_BARREL_HEIGHT: f32 = CELL_SIZE;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

#[derive(Deserialize)]
struct Entity {
    x: f32,
    y: f32,
    id: u8,
}

#[derive(Deserialize)]
struct LevelData {
    buildings: Vec<Entity>,
    cannons: Vec<Entity>,
    planes: Vec<Entity>,
    enemy_missiles: Vec<Entity>,
    ground: Vec<Entity>,
}

#[derive(PartialEq)]
struct Missile {
    x: f32,
    y: f32,
    direction: Vec2,
    speed: f32,
    trail_length: f32,
    should_destroy: bool,
}

impl Missile {
    fn new(x: f32, y: f32, direction: Vec2, speed: f32) -> Missile {
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
struct Building {
    x: f32,
    y: f32,
    size: Vec2,
    texture: Rc<Texture2D>,
    should_destroy: bool,
}

#[derive(PartialEq)]
struct Plane {
    x: f32,
    y: f32,
    direction: Vec2,
    speed: f32,
    size: Vec2,
    should_destroy: bool,
}

#[derive(Clone, Copy)]
struct Cannon {
    x: f32,
    y: f32,
    target: Vec2,
    ammo: u32,
}

struct Crosshair {
    x: f32,
    y: f32,
    should_destroy: bool,
    missile_index: usize,
}

struct Game {
    buildings: Vec<Building>,
    planes: Vec<Plane>,
    enemy_missiles_spawnpoints: Vec<Entity>,
    ground_entities: Vec<Entity>,
    enemy_missiles: Vec<Missile>,
    player_missiles: Vec<Missile>,
    cannons: Vec<Cannon>,
    plane_texture: Rc<Texture2D>,
    building_textures: Vec<Rc<Texture2D>>,
    missile_texture: Rc<Texture2D>,
    cannon_base_texture: Rc<Texture2D>,
    cannon_barrel_texture: Rc<Texture2D>,
    ground_texture: Rc<Texture2D>,
    missile_fire_sound: Rc<audio::Sound>,
    explosion_sound: Rc<audio::Sound>,
    enemy_missile_sound: Rc<audio::Sound>,
    crosshairs: Vec<Crosshair>,
    camera: Camera2D,
    game_over: bool,
    score: i32,
}

fn draw_missile(
    x: f32,
    y: f32,
    size: f32,
    direction: Vec2,
    color: Color,
    missile_texture: &Texture2D
) {
    let rotation = direction.y.atan2(direction.x);
    draw_texture_ex(missile_texture, x, y, color, DrawTextureParams {
        dest_size: Some(vec2(size, size / 2.0)),
        rotation,
        ..Default::default()
    });
}

fn draw_trail(x: f32, y: f32, length: f32, direction: Vec2) {
    draw_line(x, y, x + direction.x * length, y + direction.y * length, 1.0, WHITE);
}

fn draw_x_crosshair(x: f32, y: f32, size: f32, color: Color) {
    draw_line(x - size, y - size, x + size, y + size, 1.0, color);
    draw_line(x + size, y - size, x - size, y + size, 1.0, color);
}

fn draw_cannon(
    x: f32,
    y: f32,
    target: Vec2,
    ammo: u32,
    base_texture: &Texture2D,
    barrel_texture: &Texture2D
) {
    let direction = target - vec2(x, y);
    let rotation = (direction.y.atan2(direction.x).to_degrees() + 45.0).to_radians();
    draw_texture_ex(base_texture, x, y, WHITE, DrawTextureParams {
        dest_size: Some(vec2(CANNON_BASE_WIDTH, CANNON_BASE_HEIGHT)),
        ..Default::default()
    });
    draw_texture_ex(
        barrel_texture,
        x + 8.0,
        y - CANNON_BARREL_HEIGHT / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(CANNON_BARREL_WIDTH, CANNON_BARREL_HEIGHT)),
            rotation,
            ..Default::default()
        }
    );
    let mut ammo_text = format!("{}", ammo);
    if ammo == 0 {
        ammo_text = "OUT".to_string();
    }
    draw_text(&ammo_text, x - 10.0, y + 10.0, 20.0, WHITE);
}

fn draw_building(x: f32, y: f32, size: Vec2, color: Color, building_texture: &Texture2D) {
    draw_texture_ex(building_texture, x, y, color, DrawTextureParams {
        dest_size: Some(size),
        ..Default::default()
    });
}

fn draw_ground(x: f32, y: f32, size: f32, color: Color, ground_texture: &Texture2D) {
    draw_texture_ex(ground_texture, x, y, color, DrawTextureParams {
        dest_size: Some(vec2(size, size)),
        ..Default::default()
    });
}

fn draw_ground_entities(ground_entities: &Vec<Entity>, ground_texture: &Texture2D) {
    for ground_entity in ground_entities {
        draw_ground(ground_entity.x, ground_entity.y, 50.0, WHITE, ground_texture);
    }
}

fn clean_missiles_out_of_window(missiles: &mut Vec<Missile>) {
    for missile in missiles {
        if
            missile.x < 0.0 ||
            missile.x > screen_width() ||
            missile.y < 0.0 ||
            missile.y > screen_height()
        {
            missile.should_destroy = true;
        }
    }
}

fn update_missile(missile: &mut Missile) {
    missile.x += missile.direction.x * missile.speed;
    missile.y += missile.direction.y * missile.speed;
    missile.trail_length += missile.speed;
}

fn update_plane(plane: &mut Plane) {
    plane.x += plane.direction.x * plane.speed;
    plane.y += plane.direction.y * plane.speed;
}

fn update_cannon(cannon: &mut Cannon, closest_cannon: Option<Cannon>, cam: &Camera2D) {
    let mouse_position = mouse_position();
    let world_position = cam.screen_to_world(vec2(mouse_position.0, mouse_position.1));
    if let Some(closest_cannon) = closest_cannon {
        if closest_cannon.x == cannon.x && closest_cannon.y == cannon.y {
            cannon.target = world_position;
        }
    }
}

fn update_crosshairs(crosshairs: &mut Vec<Crosshair>, missiles: &mut Vec<Missile>) {
    for crosshair in crosshairs {
        if crosshair.should_destroy {
            continue;
        }

        if
            crosshair.missile_index >= missiles.len() ||
            missiles[crosshair.missile_index].should_destroy
        {
            crosshair.should_destroy = true;
            continue;
        }
    }
}

fn fire_missile(
    missiles: &mut Vec<Missile>,
    x: f32,
    y: f32,
    cannon: &mut Cannon,
    sfx: &audio::Sound
) -> Option<usize> {
    if cannon.ammo == 0 {
        return None;
    }

    let direction = cannon.target - vec2(x, y);
    let direction = direction.normalize();

    let missile = Missile::new(x, y, direction, 2.5);
    play_sound_once(sfx);
    missiles.push(missile);
    cannon.ammo -= 1;

    Some(missiles.len() - 1)
}

fn spawn_crosshair(game: &mut Game, missile_index: Option<usize>) {
    if missile_index.is_none() {
        return;
    }

    let mouse_position = mouse_position();
    let world_position = game.camera.screen_to_world(vec2(mouse_position.0, mouse_position.1));
    let crosshair_position = vec2(world_position.x, world_position.y);
    let crosshair = Crosshair {
        x: crosshair_position.x,
        y: crosshair_position.y,
        missile_index: missile_index.unwrap(),
        should_destroy: false,
    };
    game.crosshairs.push(crosshair);
}

fn get_closeset_cannon_no_ref(cannons: &Vec<Cannon>, cam: &Camera2D) -> Option<Cannon> {
    let mouse_position = mouse_position();
    let mouse_position = vec2(mouse_position.0, mouse_position.1);
    let world_position = cam.screen_to_world(mouse_position);
    let mut closest_cannon: Option<Cannon> = None;
    let mut closest_distance = 100000.0;
    for cannon in cannons {
        let distance = world_position.distance(vec2(cannon.x, cannon.y));
        if distance < closest_distance {
            closest_distance = distance;
            closest_cannon = Some(cannon.clone());
        }
    }

    closest_cannon
}

fn get_closest_cannon_mut<'a>(
    cannons: &'a mut Vec<Cannon>,
    cam: &'a Camera2D
) -> Option<&'a mut Cannon> {
    let mouse_position = mouse_position();
    let mouse_position = vec2(mouse_position.0, mouse_position.1);
    let world_position = cam.screen_to_world(mouse_position);
    let mut closest_cannon: Option<&mut Cannon> = None;
    let mut closest_distance = 100000.0;
    for cannon in cannons {
        let distance = world_position.distance(vec2(cannon.x, cannon.y));
        if distance < closest_distance {
            closest_distance = distance;
            closest_cannon = Some(cannon);
        }
    }

    closest_cannon
}

fn aabb_collision(x1: f32, y1: f32, size1: Vec2, x2: f32, y2: f32, size2: Vec2) -> bool {
    if
        x1 - size1.x / 2.0 < x2 + size2.y / 2.0 &&
        x1 + size1.x / 2.0 > x2 - size2.y / 2.0 &&
        y1 - size1.x / 2.0 < y2 + size2.y / 2.0 &&
        y1 + size1.x / 2.0 > y2 - size2.y / 2.0
    {
        return true;
    }

    false
}

fn missile_hit_building(missile: &Missile, building: &Building) -> bool {
    return aabb_collision(
        missile.x,
        missile.y,
        Vec2::new(MISSILE_SIZE, MISSILE_SIZE / 2.0),
        building.x + building.size.x / 2.0,
        building.y + building.size.y / 2.0,
        building.size
    );
}

fn missile_hit_plane(missile: &Missile, plane: &Plane) -> bool {
    return aabb_collision(
        missile.x,
        missile.y,
        Vec2::new(MISSILE_SIZE, MISSILE_SIZE / 2.0),
        plane.x,
        plane.y,
        plane.size
    );
}

fn missile_hit_missile(missile1: &Missile, missile2: &Missile) -> bool {
    return aabb_collision(
        missile1.x,
        missile1.y,
        Vec2::new(MISSILE_SIZE, MISSILE_SIZE / 2.0),
        missile2.x,
        missile2.y,
        Vec2::new(MISSILE_SIZE, MISSILE_SIZE / 2.0)
    );
}

fn handle_missile_building_collision(
    buildings: &mut Vec<Building>,
    missile: &mut Missile,
    sfx: &audio::Sound
) {
    for building in buildings {
        if missile_hit_building(missile, building) {
            building.should_destroy = true;
            missile.should_destroy = true;
            play_sound_once(sfx);
        }
    }
}

fn handle_missile_plane_collision(
    planes: &mut Vec<Plane>,
    missile: &mut Missile,
    score: &mut i32,
    sfx: &audio::Sound
) {
    for plane in planes {
        if missile_hit_plane(missile, plane) {
            plane.should_destroy = true;
            missile.should_destroy = true;
            play_sound_once(sfx);
            *score -= 10;
        }
    }
}

fn handle_missile_missile_collision(
    enemy_missiles: &mut Vec<Missile>,
    missile: &mut Missile,
    score: &mut i32,
    sfx: &audio::Sound
) {
    for enemy_missile in enemy_missiles {
        if missile_hit_missile(missile, enemy_missile) {
            enemy_missile.should_destroy = true;
            missile.should_destroy = true;
            play_sound_once(sfx);
            *score += 1;
        }
    }
}

fn handle_collisions(game: &mut Game) {
    for missile in &mut game.enemy_missiles {
        handle_missile_building_collision(&mut game.buildings, missile, &game.explosion_sound);
    }

    for missile in &mut game.player_missiles {
        handle_missile_plane_collision(
            &mut game.planes,
            missile,
            &mut game.score,
            &game.explosion_sound
        );
        handle_missile_missile_collision(
            &mut game.enemy_missiles,
            missile,
            &mut game.score,
            &game.explosion_sound
        );
    }
}

fn update_game(game: &mut Game) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let closest_cannon = get_closest_cannon_mut(&mut game.cannons, &game.camera);
        if let Some(cannon) = closest_cannon {
            if cannon.ammo == 0 {
                return;
            }
            let missile_index = fire_missile(
                &mut game.player_missiles,
                cannon.x + 16.0,
                cannon.y,
                cannon,
                &game.missile_fire_sound
            );
            spawn_crosshair(game, missile_index);
        }
    }

    for missile in &mut game.enemy_missiles {
        update_missile(missile);
    }

    for missile in &mut game.player_missiles {
        update_missile(missile);
    }

    for plane in &mut game.planes {
        update_plane(plane);
    }

    let closest_cannon = get_closeset_cannon_no_ref(&game.cannons, &game.camera);

    for cannon in &mut game.cannons {
        update_cannon(cannon, closest_cannon, &game.camera);
    }

    handle_collisions(game);
    update_crosshairs(&mut game.crosshairs, &mut game.player_missiles);
    cleanup(game);

    if game.buildings.len() == 0 {
        game.game_over = true;
    }
}

fn draw_buildings(buildings: &Vec<Building>) {
    for building in buildings {
        draw_building(building.x, building.y, building.size, BUILDING_COLOR, &building.texture);
        // draw_aabb(
        //     building.x + building.size / 2.0,
        //     building.y + building.size / 2.0,
        //     building.size,
        //     BLUE
        // );
    }
}

fn draw_planes(planes: &Vec<Plane>, plane_texture: &Texture2D) {
    for plane in planes {
        let should_flip = plane.direction.x > 0.0;
        draw_texture_ex(plane_texture, plane.x, plane.y, WHITE, DrawTextureParams {
            dest_size: Some(plane.size),
            flip_x: should_flip,
            ..Default::default()
        });
    }
}

fn draw_enemy_missiles(enemy_missiles: &Vec<Missile>, missile_texture: &Texture2D) {
    for missile in enemy_missiles {
        draw_trail(
            missile.x + MISSILE_SIZE / 2.0,
            missile.y,
            missile.trail_length,
            -missile.direction
        );
        draw_missile(
            missile.x,
            missile.y,
            MISSILE_SIZE,
            missile.direction,
            ENEMY_COLOR,
            missile_texture
        );
        //draw_aabb(missile.x, missile.y, MISSILE_SIZE, BLUE);
    }
}

fn draw_player_missiles(player_missiles: &Vec<Missile>, missile_texture: &Texture2D) {
    for missile in player_missiles {
        draw_trail(
            missile.x + MISSILE_SIZE / 2.0,
            missile.y,
            missile.trail_length,
            -missile.direction
        );
        draw_missile(missile.x, missile.y, MISSILE_SIZE, missile.direction, WHITE, missile_texture);
        //draw_aabb(missile.x, missile.y, MISSILE_SIZE, BLUE);
    }
}

fn draw_cannons(
    cannons: &Vec<Cannon>,
    cannon_base_texture: &Texture2D,
    cannon_barrel_texture: &Texture2D
) {
    for cannon in cannons {
        draw_cannon(
            cannon.x,
            cannon.y,
            cannon.target,
            cannon.ammo,
            cannon_base_texture,
            cannon_barrel_texture
        );
    }
}

fn draw_crosshairs(crosshairs: &Vec<Crosshair>) {
    for crosshair in crosshairs {
        if crosshair.should_destroy {
            continue;
        }

        draw_x_crosshair(crosshair.x, crosshair.y, 10.0, PLAYER_COLOR);
    }
}

fn draw_score(score: i32) {
    let score_text = format!("Score: {}", score);
    draw_text(&score_text, 10.0, 20.0, 30.0, WHITE);
}

fn draw_game(game: &Game) {
    draw_ground_entities(&game.ground_entities, &game.ground_texture);
    draw_buildings(&game.buildings);
    draw_planes(&game.planes, &game.plane_texture);
    draw_cannons(&game.cannons, &game.cannon_base_texture, &game.cannon_barrel_texture);
    draw_crosshairs(&game.crosshairs);
    draw_enemy_missiles(&game.enemy_missiles, &game.missile_texture);
    draw_player_missiles(&game.player_missiles, &game.missile_texture);
    draw_score(game.score);
}

fn cleanup_buildings(game: &mut Game) {
    game.buildings.retain(|building| !building.should_destroy);
}

fn cleanup_planes(game: &mut Game) {
    game.planes.retain(|plane| !plane.should_destroy);
}

fn cleanup_enemy_missiles(game: &mut Game) {
    game.enemy_missiles.retain(|missile| !missile.should_destroy);
}

fn cleanup_player_missiles(game: &mut Game) {
    game.player_missiles.retain(|missile| !missile.should_destroy);
}

fn cleanup_crosshairs(game: &mut Game) {
    game.crosshairs.retain(|crosshair| !crosshair.should_destroy);
}

fn cleanup(game: &mut Game) {
    cleanup_buildings(game);
    cleanup_planes(game);
    cleanup_crosshairs(game);
    cleanup_enemy_missiles(game);
    cleanup_player_missiles(game);
    clean_missiles_out_of_window(&mut game.enemy_missiles);
    clean_missiles_out_of_window(&mut game.player_missiles);
}

fn spawn_enemy_missile(game: &mut Game) {
    let spawnpoint_index = rand::gen_range(0, game.enemy_missiles_spawnpoints.len());
    if spawnpoint_index >= game.enemy_missiles_spawnpoints.len() {
        return;
    }
    let spawnpoint = &game.enemy_missiles_spawnpoints[spawnpoint_index];
    let x = spawnpoint.x;
    let y = spawnpoint.y;
    let direction = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(0.2, 1.0));
    let direction = direction.normalize();
    let missile = Missile::new(x, y, direction, 1.0);
    play_sound_once(&game.enemy_missile_sound);
    game.enemy_missiles.push(missile);
}

fn spawn_cannon(game: &mut Game, x: f32, y: f32) {
    let cannon = Cannon {
        x,
        y,
        target: vec2(0.0, 0.0),
        ammo: 10,
    };

    game.cannons.push(cannon);
}

fn spawn_building(game: &mut Game, x: f32, y: f32, id: u8) {
    let building = Building {
        x,
        y,
        size: vec2(64.0, 64.0),
        texture: game.building_textures[id as usize].clone(),
        should_destroy: false,
    };

    game.buildings.push(building);
}

fn spawn_enemy_missiles(game: &mut Game) {
    let num_missiles = rand::gen_range(10, 15);
    for _ in 0..num_missiles {
        spawn_enemy_missile(game);
    }
}

fn draw_aabb(x: f32, y: f32, size: f32, color: Color) {
    draw_rectangle_lines(x - size / 2.0, y - size / 2.0, size, size, 2.0, color);
}

fn draw_background(texture: &Texture2D) {
    // draw texture centered on screen and scale it to screen size (without changing aspect ratio)
    let screen_width = screen_width();
    let screen_height = screen_height();
    let texture_width = texture.width();
    let texture_height = texture.height();
    let scale = screen_width / texture_width;
    let x = screen_width / 2.0 - (texture_width / 2.0) * scale;
    let y = screen_height / 2.0 - (texture_height / 2.0) * scale;
    draw_texture_ex(texture, x, y, WHITE, DrawTextureParams {
        dest_size: Some(vec2(texture_width * scale, texture_height * scale)),
        ..Default::default()
    });
}

fn handle_resize(last_screen_width: f32, last_screen_height: f32, game: &mut Game) {
    let screen_width = screen_width();
    let screen_height = screen_height();
    if screen_width != last_screen_width || screen_height != last_screen_height {
        game.camera.zoom = vec2((1.0 / screen_width) * 2.0, (1.0 / screen_height) * 2.0);
    }
}

fn load_level_from_file(path: &str) -> LevelData {
    let level_data = Assets::get(path).unwrap().data;
    let level_data: LevelData = serde_json::from_slice(&level_data).unwrap();
    level_data
}

fn load_level(game: &mut Game) {
    let level_data = load_level_from_file("level.json");
    for building in level_data.buildings {
        spawn_building(game, building.x, building.y, building.id);
    }

    for cannon in level_data.cannons {
        spawn_cannon(game, cannon.x, cannon.y);
    }

    for plane in level_data.planes {
        let plane = Plane {
            x: plane.x,
            y: plane.y,
            direction: get_plane_direction(plane.x),
            speed: 1.0,
            size: vec2(PLANE_WIDTH, PLANE_HEIGHT),
            should_destroy: false,
        };

        game.planes.push(plane);
    }

    for enemy_missile_spawnpoint in level_data.enemy_missiles {
        game.enemy_missiles_spawnpoints.push(enemy_missile_spawnpoint);
    }

    for ground_entity in level_data.ground {
        game.ground_entities.push(ground_entity);
    }
}

fn get_plane_direction(x: f32) -> Vec2 {
    if x < screen_width() * 0.5 {
        return vec2(1.0, 0.0);
    }

    return vec2(-1.0, 0.0);
}

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
    rand::srand(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u64);
    let building1_texture = Texture2D::from_file_with_format(
        &Assets::get("building_1.png").unwrap().data,
        Some(ImageFormat::Png)
    );
    let building2_texture = Texture2D::from_file_with_format(
        &Assets::get("building_2.png").unwrap().data,
        Some(ImageFormat::Png)
    );
    let building3_texture = Texture2D::from_file_with_format(
        &Assets::get("building_3.png").unwrap().data,
        Some(ImageFormat::Png)
    );
    let plane_texture = Texture2D::from_file_with_format(
        &Assets::get("plane.png").unwrap().data,
        Some(ImageFormat::Png)
    );
    let missile_texture = Texture2D::from_file_with_format(
        &Assets::get("missile.png").unwrap().data,
        Some(ImageFormat::Png)
    );
    let background_texture = Texture2D::from_file_with_format(
        &Assets::get("background.png").unwrap().data,
        Some(ImageFormat::Png)
    );
    let cannon_base_texture = Texture2D::from_file_with_format(
        &Assets::get("missile_launcher_part_1.png").unwrap().data,
        Some(ImageFormat::Png)
    );
    let cannon_barrel_texture = Texture2D::from_file_with_format(
        &Assets::get("missile_launcher_part_2.png").unwrap().data,
        Some(ImageFormat::Png)
    );
    let ground_texture = Texture2D::from_file_with_format(
        &Assets::get("ground.png").unwrap().data,
        Some(ImageFormat::Png)
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
    let missile_fire_sound = audio
        ::load_sound_from_bytes(&Assets::get("missile_fire.ogg").unwrap().data).await
        .unwrap();
    let explosion_sound = audio
        ::load_sound_from_bytes(&Assets::get("explosion.ogg").unwrap().data).await
        .unwrap();
    let enemy_missile_sound = audio
        ::load_sound_from_bytes(&Assets::get("enemy_missile.ogg").unwrap().data).await
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
