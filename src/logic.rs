use crate::data::{
    Assets, Building, Cannon, Crosshair, Game, LevelData, Missile, Plane, MISSILE_SIZE,
    PLANE_HEIGHT, PLANE_WIDTH,
};
use macroquad::{
    audio::{self, play_sound_once},
    prelude::*,
};

pub fn clean_missiles_out_of_window(missiles: &mut Vec<Missile>) {
    for missile in missiles {
        if missile.x < 0.0
            || missile.x > screen_width()
            || missile.y < 0.0
            || missile.y > screen_height()
        {
            missile.should_destroy = true;
        }
    }
}

pub fn update_missile(missile: &mut Missile) {
    missile.x += missile.direction.x * missile.speed;
    missile.y += missile.direction.y * missile.speed;
    missile.trail_length += missile.speed;
}

pub fn update_plane(plane: &mut Plane) {
    plane.x += plane.direction.x * plane.speed;
    plane.y += plane.direction.y * plane.speed;
}

pub fn update_cannon(cannon: &mut Cannon, closest_cannon: Option<Cannon>, cam: &Camera2D) {
    let mouse_position = mouse_position();
    let world_position = cam.screen_to_world(vec2(mouse_position.0, mouse_position.1));
    if let Some(closest_cannon) = closest_cannon {
        if closest_cannon.x == cannon.x && closest_cannon.y == cannon.y {
            cannon.target = world_position;
        }
    }
}

pub fn update_crosshairs(crosshairs: &mut Vec<Crosshair>, missiles: &mut Vec<Missile>) {
    for crosshair in crosshairs {
        if crosshair.should_destroy {
            continue;
        }

        if crosshair.missile_index >= missiles.len()
            || missiles[crosshair.missile_index].should_destroy
        {
            crosshair.should_destroy = true;
            continue;
        }
    }
}

pub fn fire_missile(
    missiles: &mut Vec<Missile>,
    x: f32,
    y: f32,
    cannon: &mut Cannon,
    sfx: &audio::Sound,
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

pub fn spawn_crosshair(game: &mut Game, missile_index: Option<usize>) {
    if missile_index.is_none() {
        return;
    }

    let mouse_position = mouse_position();
    let world_position = game
        .camera
        .screen_to_world(vec2(mouse_position.0, mouse_position.1));
    let crosshair_position = vec2(world_position.x, world_position.y);
    let crosshair = Crosshair {
        x: crosshair_position.x,
        y: crosshair_position.y,
        missile_index: missile_index.unwrap(),
        should_destroy: false,
    };
    game.crosshairs.push(crosshair);
}

pub fn get_closeset_cannon_no_ref(cannons: &Vec<Cannon>, cam: &Camera2D) -> Option<Cannon> {
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

pub fn get_closest_cannon_mut<'a>(
    cannons: &'a mut Vec<Cannon>,
    cam: &'a Camera2D,
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

pub fn aabb_collision(x1: f32, y1: f32, size1: Vec2, x2: f32, y2: f32, size2: Vec2) -> bool {
    if x1 - size1.x / 2.0 < x2 + size2.y / 2.0
        && x1 + size1.x / 2.0 > x2 - size2.y / 2.0
        && y1 - size1.x / 2.0 < y2 + size2.y / 2.0
        && y1 + size1.x / 2.0 > y2 - size2.y / 2.0
    {
        return true;
    }

    false
}

pub fn missile_hit_building(missile: &Missile, building: &Building) -> bool {
    return aabb_collision(
        missile.x,
        missile.y,
        Vec2::new(MISSILE_SIZE, MISSILE_SIZE / 2.0),
        building.x + building.size.x / 2.0,
        building.y + building.size.y / 2.0,
        building.size,
    );
}

pub fn missile_hit_plane(missile: &Missile, plane: &Plane) -> bool {
    return aabb_collision(
        missile.x,
        missile.y,
        Vec2::new(MISSILE_SIZE, MISSILE_SIZE / 2.0),
        plane.x,
        plane.y,
        plane.size,
    );
}

pub fn missile_hit_missile(missile1: &Missile, missile2: &Missile) -> bool {
    return aabb_collision(
        missile1.x,
        missile1.y,
        Vec2::new(MISSILE_SIZE, MISSILE_SIZE / 2.0),
        missile2.x,
        missile2.y,
        Vec2::new(MISSILE_SIZE, MISSILE_SIZE / 2.0),
    );
}

pub fn handle_missile_building_collision(
    buildings: &mut Vec<Building>,
    missile: &mut Missile,
    sfx: &audio::Sound,
) {
    for building in buildings {
        if missile_hit_building(missile, building) {
            building.should_destroy = true;
            missile.should_destroy = true;
            play_sound_once(sfx);
        }
    }
}

pub fn handle_missile_plane_collision(
    planes: &mut Vec<Plane>,
    missile: &mut Missile,
    score: &mut i32,
    sfx: &audio::Sound,
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

pub fn handle_missile_missile_collision(
    enemy_missiles: &mut Vec<Missile>,
    missile: &mut Missile,
    score: &mut i32,
    sfx: &audio::Sound,
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

pub fn handle_collisions(game: &mut Game) {
    for missile in &mut game.enemy_missiles {
        handle_missile_building_collision(&mut game.buildings, missile, &game.explosion_sound);
    }

    for missile in &mut game.player_missiles {
        handle_missile_plane_collision(
            &mut game.planes,
            missile,
            &mut game.score,
            &game.explosion_sound,
        );
        handle_missile_missile_collision(
            &mut game.enemy_missiles,
            missile,
            &mut game.score,
            &game.explosion_sound,
        );
    }
}

pub fn update_game(game: &mut Game) {
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
                &game.missile_fire_sound,
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

pub fn cleanup_buildings(game: &mut Game) {
    game.buildings.retain(|building| !building.should_destroy);
}

pub fn cleanup_planes(game: &mut Game) {
    game.planes.retain(|plane| !plane.should_destroy);
}

pub fn cleanup_enemy_missiles(game: &mut Game) {
    game.enemy_missiles
        .retain(|missile| !missile.should_destroy);
}

pub fn cleanup_player_missiles(game: &mut Game) {
    game.player_missiles
        .retain(|missile| !missile.should_destroy);
}

pub fn cleanup_crosshairs(game: &mut Game) {
    game.crosshairs
        .retain(|crosshair| !crosshair.should_destroy);
}

pub fn cleanup(game: &mut Game) {
    cleanup_buildings(game);
    cleanup_planes(game);
    cleanup_crosshairs(game);
    cleanup_enemy_missiles(game);
    cleanup_player_missiles(game);
    clean_missiles_out_of_window(&mut game.enemy_missiles);
    clean_missiles_out_of_window(&mut game.player_missiles);
}

pub fn spawn_enemy_missile(game: &mut Game) {
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

pub fn spawn_cannon(game: &mut Game, x: f32, y: f32) {
    let cannon = Cannon {
        x,
        y,
        target: vec2(0.0, 0.0),
        ammo: 10,
    };

    game.cannons.push(cannon);
}

pub fn spawn_building(game: &mut Game, x: f32, y: f32, id: u8) {
    let building = Building {
        x,
        y,
        size: vec2(64.0, 64.0),
        texture: game.building_textures[id as usize].clone(),
        should_destroy: false,
    };

    game.buildings.push(building);
}

pub fn spawn_enemy_missiles(game: &mut Game) {
    let num_missiles = rand::gen_range(10, 15);
    for _ in 0..num_missiles {
        spawn_enemy_missile(game);
    }
}

pub fn handle_resize(last_screen_width: f32, last_screen_height: f32, game: &mut Game) {
    let screen_width = screen_width();
    let screen_height = screen_height();
    if screen_width != last_screen_width || screen_height != last_screen_height {
        game.camera.zoom = vec2((1.0 / screen_width) * 2.0, (1.0 / screen_height) * 2.0);
    }
}

pub fn load_level_from_file(path: &str) -> LevelData {
    let level_data = Assets::get(path).unwrap().data;
    let level_data: LevelData = serde_json::from_slice(&level_data).unwrap();
    level_data
}

pub fn load_level(game: &mut Game) {
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
        game.enemy_missiles_spawnpoints
            .push(enemy_missile_spawnpoint);
    }

    for ground_entity in level_data.ground {
        game.ground_entities.push(ground_entity);
    }
}

pub fn get_plane_direction(x: f32) -> Vec2 {
    if x < screen_width() * 0.5 {
        return vec2(1.0, 0.0);
    }

    return vec2(-1.0, 0.0);
}
