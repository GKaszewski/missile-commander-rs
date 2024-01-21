#![windows_subsystem = "windows"]
use macroquad::{ prelude::*, audio::{ self, play_sound_once } };
use std::{ time::{ SystemTime, UNIX_EPOCH }, rc::Rc };

const ENEMY_COLOR: Color = RED;
const PLAYER_COLOR: Color = GREEN;
const BUILDING_COLOR: Color = WHITE;
const NUM_BUILDINGS: u32 = 3;
const MISSILE_SIZE: f32 = 10.0;

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
    size: f32,
    texture: Rc<Texture2D>,
    should_destroy: bool,
}

#[derive(PartialEq)]
struct Plane {
    x: f32,
    y: f32,
    direction: Vec2,
    speed: f32,
    size: f32,
    should_destroy: bool,
}

#[derive(Clone, Copy)]
struct Cannon {
    x: f32,
    y: f32,
    size: f32,
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
    enemy_missiles: Vec<Missile>,
    player_missiles: Vec<Missile>,
    cannons: Vec<Cannon>,
    plane_texture: Rc<Texture2D>,
    building_texture: Rc<Texture2D>,
    missile_fire_sound: Rc<audio::Sound>,
    explosion_sound: Rc<audio::Sound>,
    enemy_missile_sound: Rc<audio::Sound>,
    crosshairs: Vec<Crosshair>,
    game_over: bool,
    score: i32,
}

fn draw_missile(x: f32, y: f32, size: f32, direction: Vec2, color: Color) {
    let rotation = direction.y.atan2(direction.x).to_degrees();
    draw_poly(x, y, 3, size, rotation, color);
}

fn draw_trail(x: f32, y: f32, length: f32, direction: Vec2) {
    draw_line(x, y, x + direction.x * length, y + direction.y * length, 1.0, WHITE);
}

fn draw_x_crosshair(x: f32, y: f32, size: f32, color: Color) {
    draw_line(x - size, y - size, x + size, y + size, 1.0, color);
    draw_line(x + size, y - size, x - size, y + size, 1.0, color);
}

fn draw_cannon(x: f32, y: f32, size: f32, target: Vec2, color: Color, ammo: u32) {
    let direction = target - vec2(x, y);
    let rotation = direction.y.atan2(direction.x).to_degrees();
    draw_circle_lines(x, y, size, 1.0, color);
    draw_poly_lines(x, y - size, 3, size, rotation, 1.0, color);
    // draw ammo count
    let mut ammo_text = format!("{}", ammo);
    if ammo == 0 {
        ammo_text = "OUT".to_string();
    }
    draw_text(&ammo_text, x - 10.0, y + 10.0, 20.0, WHITE);
}

fn draw_building(x: f32, y: f32, size: f32, color: Color, building_texture: &Texture2D) {
    draw_texture_ex(building_texture, x, y, color, DrawTextureParams {
        dest_size: Some(vec2(size, size)),
        ..Default::default()
    });
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

fn update_cannon(cannon: &mut Cannon, closest_cannon: Option<Cannon>) {
    let mouse_position = mouse_position();
    let target = vec2(mouse_position.0, mouse_position.1);
    if let Some(closest_cannon) = closest_cannon {
        if closest_cannon.x == cannon.x && closest_cannon.y == cannon.y {
            cannon.target = target;
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
    let crosshair_position = vec2(mouse_position.0, mouse_position.1);
    let crosshair = Crosshair {
        x: crosshair_position.x,
        y: crosshair_position.y,
        missile_index: missile_index.unwrap(),
        should_destroy: false,
    };
    game.crosshairs.push(crosshair);
}

fn get_closeset_cannon_no_ref(cannons: &Vec<Cannon>) -> Option<Cannon> {
    let mouse_position = mouse_position();
    let mouse_position = vec2(mouse_position.0, mouse_position.1);
    let mut closest_cannon: Option<Cannon> = None;
    let mut closest_distance = 100000.0;
    for cannon in cannons {
        let distance = mouse_position.distance(vec2(cannon.x, cannon.y));
        if distance < closest_distance {
            closest_distance = distance;
            closest_cannon = Some(cannon.clone());
        }
    }

    closest_cannon
}

fn get_closest_cannon_mut(cannons: &mut Vec<Cannon>) -> Option<&mut Cannon> {
    let mouse_position = mouse_position();
    let mouse_position = vec2(mouse_position.0, mouse_position.1);
    let mut closest_cannon: Option<&mut Cannon> = None;
    let mut closest_distance = 100000.0;
    for cannon in cannons {
        let distance = mouse_position.distance(vec2(cannon.x, cannon.y));
        if distance < closest_distance {
            closest_distance = distance;
            closest_cannon = Some(cannon);
        }
    }

    closest_cannon
}

fn aabb_collision(x1: f32, y1: f32, size1: f32, x2: f32, y2: f32, size2: f32) -> bool {
    if
        x1 - size1 / 2.0 < x2 + size2 / 2.0 &&
        x1 + size1 / 2.0 > x2 - size2 / 2.0 &&
        y1 - size1 / 2.0 < y2 + size2 / 2.0 &&
        y1 + size1 / 2.0 > y2 - size2 / 2.0
    {
        return true;
    }

    false
}

fn missile_hit_building(missile: &Missile, building: &Building) -> bool {
    return aabb_collision(
        missile.x,
        missile.y,
        MISSILE_SIZE,
        building.x + building.size / 2.0,
        building.y + building.size / 2.0,
        building.size
    );
}

fn missile_hit_plane(missile: &Missile, plane: &Plane) -> bool {
    return aabb_collision(missile.x, missile.y, MISSILE_SIZE, plane.x, plane.y, plane.size);
}

fn missile_hit_missile(missile1: &Missile, missile2: &Missile) -> bool {
    return aabb_collision(
        missile1.x,
        missile1.y,
        MISSILE_SIZE,
        missile2.x,
        missile2.y,
        MISSILE_SIZE
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
        let closest_cannon = get_closest_cannon_mut(&mut game.cannons);
        if let Some(cannon) = closest_cannon {
            if cannon.ammo == 0 {
                return;
            }
            let missile_index = fire_missile(
                &mut game.player_missiles,
                cannon.x,
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

    let closest_cannon = get_closeset_cannon_no_ref(&game.cannons);

    for cannon in &mut game.cannons {
        update_cannon(cannon, closest_cannon);
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
        draw_texture_ex(plane_texture, plane.x, plane.y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(plane.size, plane.size)),
            ..Default::default()
        });
    }
}

fn draw_enemy_missiles(enemy_missiles: &Vec<Missile>) {
    for missile in enemy_missiles {
        draw_trail(missile.x, missile.y, missile.trail_length, -missile.direction);
        draw_missile(missile.x, missile.y, 10.0, missile.direction, ENEMY_COLOR);
        // draw_aabb(missile.x, missile.y, MISSILE_SIZE, BLUE);
    }
}

fn draw_player_missiles(player_missiles: &Vec<Missile>) {
    for missile in player_missiles {
        draw_trail(missile.x, missile.y, missile.trail_length, -missile.direction);
        draw_missile(missile.x, missile.y, 10.0, missile.direction, PLAYER_COLOR);
        // draw_aabb(missile.x, missile.y, MISSILE_SIZE, BLUE);
    }
}

fn draw_cannons(cannons: &Vec<Cannon>) {
    for cannon in cannons {
        draw_cannon(cannon.x, cannon.y, cannon.size, cannon.target, PLAYER_COLOR, cannon.ammo);
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
    draw_buildings(&game.buildings);
    draw_planes(&game.planes, &game.plane_texture);
    draw_cannons(&game.cannons);
    draw_crosshairs(&game.crosshairs);
    draw_enemy_missiles(&game.enemy_missiles);
    draw_player_missiles(&game.player_missiles);
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
    let x = rand::gen_range(5.0, screen_width() - 5.0);
    let y = 0.0;
    let direction = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(0.2, 1.0));
    let direction = direction.normalize();
    let missile = Missile::new(x, y, direction, 1.0);
    play_sound_once(&game.enemy_missile_sound);
    game.enemy_missiles.push(missile);
}

fn spawn_plane(game: &mut Game) {
    let right = rand::gen_range(0, 2);
    let x = if right == 0 { 0.0 } else { screen_width() };
    let y = rand::gen_range(0.0, screen_height() * 0.5);
    let direction: Vec2;
    if right == 0 {
        direction = vec2(1.0, 0.0);
    } else {
        direction = vec2(-1.0, 0.0);
    }

    let plane = Plane {
        x,
        y,
        direction,
        speed: 1.0,
        size: 50.0,
        should_destroy: false,
    };

    game.planes.push(plane);
}

fn spawn_cannon(game: &mut Game, x: f32, y: f32) {
    let cannon = Cannon {
        x,
        y,
        size: 20.0,
        target: vec2(0.0, 0.0),
        ammo: 10,
    };

    game.cannons.push(cannon);
}

fn spawn_building(game: &mut Game, x: f32, y: f32) {
    let size = 50.0;
    let building = Building {
        x,
        y,
        size,
        texture: game.building_texture.clone(),
        should_destroy: false,
    };

    game.buildings.push(building);
}

fn spawn_group_buildings(game: &mut Game, gap: f32, x_start: f32, y: f32, num_buildings: u32) {
    let mut x = x_start;
    for _ in 0..num_buildings {
        spawn_building(game, x, y);
        x += gap;
    }
}

fn spawn_buildings(game: &mut Game) {
    // spawn 3 groups of buildings. Each group has 5 buildings, with a gap of 100px between them.
    // Between each group there is a gap of 200px.

    let gap = 51.0;
    let x_start = 100.0;
    let y = screen_height() - 50.0;
    let group_gap = 70.0;

    for i in 0..3 {
        let x = x_start + (i as f32) * (NUM_BUILDINGS as f32) * gap + (i as f32) * group_gap;
        spawn_group_buildings(game, gap, x, y, NUM_BUILDINGS);
    }
}

fn spawn_cannons(game: &mut Game) {
    let y = screen_height() - 50.0;
    spawn_cannon(game, 50.0, y);
    spawn_cannon(game, 300.0, y);
    spawn_cannon(game, 720.0, y);
}

fn spawn_enemy_missiles(game: &mut Game) {
    let num_missiles = rand::gen_range(10, 15);
    for _ in 0..num_missiles {
        spawn_enemy_missile(game);
    }
}

// fn draw_aabb(x: f32, y: f32, size: f32, color: Color) {
//     draw_rectangle_lines(x - size / 2.0, y - size / 2.0, size, size, 2.0, color);
// }

#[macroquad::main("Missile commander")]
async fn main() {
    rand::srand(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u64);
    let building_texture = load_texture("assets/building.png").await.unwrap();
    let plane_texture = load_texture("assets/plane.png").await.unwrap();
    building_texture.set_filter(FilterMode::Nearest);
    plane_texture.set_filter(FilterMode::Nearest);
    let missile_fire_sound = audio::load_sound("assets/missile_fire.ogg").await.unwrap();
    let explosion_sound = audio::load_sound("assets/explosion.ogg").await.unwrap();
    let enemy_missile_sound = audio::load_sound("assets/enemy_missile.ogg").await.unwrap();

    let mut game = Game {
        buildings: vec![],
        planes: vec![],
        enemy_missiles: vec![],
        player_missiles: vec![],
        cannons: vec![],
        plane_texture: Rc::new(plane_texture),
        building_texture: Rc::new(building_texture),
        missile_fire_sound: Rc::new(missile_fire_sound),
        explosion_sound: Rc::new(explosion_sound),
        enemy_missile_sound: Rc::new(enemy_missile_sound),
        crosshairs: vec![],
        score: 0,
        game_over: false,
    };
    spawn_buildings(&mut game);
    spawn_cannons(&mut game);
    spawn_enemy_missiles(&mut game);
    spawn_plane(&mut game);
    loop {
        clear_background(BLACK);
        update_game(&mut game);
        draw_game(&game);
        next_frame().await;
    }
}
