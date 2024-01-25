use macroquad::prelude::*;

const GRID_CELL_SIZE: f32 = 32.0;

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Missile Editor - Level Editor"),
        window_width: 1280,
        window_height: 720,
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

fn draw_pointer(cam: &Camera2D) {
    let mouse_pos = mouse_position();
    let mouse_pos = vec2(mouse_pos.0, mouse_pos.1);
    let mouse_pos = cam.screen_to_world(mouse_pos);

    let x = (mouse_pos.x / GRID_CELL_SIZE).floor() * GRID_CELL_SIZE;
    let y = (mouse_pos.y / GRID_CELL_SIZE).floor() * GRID_CELL_SIZE;

    draw_rectangle(x, y, GRID_CELL_SIZE, GRID_CELL_SIZE, RED);
}

#[macroquad::main(window_conf)]
async fn main() {
    let camera = Camera2D {
        zoom: vec2((1.0 / screen_width()) * 2.0, (1.0 / screen_height()) * 2.0),
        target: vec2(screen_width() / 2.0, screen_height() / 2.0),
        ..Default::default()
    };
    set_camera(&camera);
    loop {
        clear_background(LIGHTGRAY);
        draw_editor_grid();
        draw_pointer(&camera);
        next_frame().await;
    }
}
