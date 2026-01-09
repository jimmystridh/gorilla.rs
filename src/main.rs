use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::f32::consts::PI;

// Constants from the original
const OBJECT_COLOR: Color = BROWN;
const WINDOW_COLOR: Color = YELLOW;
const SUN_ATTR: Color = Color::new(1.0, 1.0, 0.0, 1.0);
const SUN_HAPPY: bool = false;
const SUN_SHOCK: bool = true;
const ARMS_DOWN: i32 = 3;
const LEFT_UP: i32 = 2;
const RIGHT_UP: i32 = 1;

// Virtual screen dimensions (original EGA)
const VIRTUAL_WIDTH: f32 = 640.0;
const VIRTUAL_HEIGHT: f32 = 350.0;
const G_HEIGHT: f32 = 25.0;
const SUN_HT: f32 = 39.0;

const BUILDING_COLORS: [Color; 4] = [
    Color::new(0.5, 0.0, 0.5, 1.0),
    Color::new(0.0, 0.5, 0.5, 1.0),
    Color::new(0.5, 0.5, 0.0, 1.0),
    Color::new(0.3, 0.3, 0.6, 1.0),
];

const EXPLOSION_COLOR: Color = RED;
const BACK_COLOR: Color = Color::new(0.0, 0.0, 0.5, 1.0);

#[derive(Clone, Copy, Default)]
struct XYPoint {
    x_coor: i32,
    y_coor: i32,
}

#[derive(Clone)]
struct Building {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color,
    windows: Vec<Window>,
}

#[derive(Clone, Copy)]
struct Window {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color,
}

struct GameState {
    gorilla_x: [f32; 2],
    gorilla_y: [f32; 2],
    last_building: usize,
    gravity: f32,
    wind: i32,
    sun_hit: bool,
    player1_name: String,
    player2_name: String,
    num_games: i32,
    total_wins: [i32; 2],
    bcoor: Vec<XYPoint>,
    buildings: Vec<Building>,
    cityscape_generated: bool,
}

impl GameState {
    fn new() -> Self {
        GameState {
            gorilla_x: [0.0; 2],
            gorilla_y: [0.0; 2],
            last_building: 0,
            gravity: 9.8,
            wind: 0,
            sun_hit: false,
            player1_name: "Player 1".to_string(),
            player2_name: "Player 2".to_string(),
            num_games: 3,
            total_wins: [0, 0],
            bcoor: vec![XYPoint::default(); 31],
            buildings: Vec::new(),
            cityscape_generated: false,
        }
    }

    fn reset_cityscape(&mut self) {
        self.buildings.clear();
        self.cityscape_generated = false;
        self.bcoor = vec![XYPoint::default(); 31];
    }
}

#[derive(PartialEq, Clone, Copy)]
enum GamePhase {
    Intro,
    GetInputs,
    GorillaIntro,
    Playing,
    GameOver,
    PlayAgain,
}

#[derive(PartialEq, Clone, Copy)]
enum InputField {
    Player1Name,
    Player2Name,
    NumGames,
    Gravity,
    ViewOrPlay,
}

#[derive(PartialEq, Clone, Copy)]
enum ShotPhase {
    InputAngle,
    InputVelocity,
    Animating,
    Done,
}

struct ShotState {
    phase: ShotPhase,
    angle: f32,
    velocity: f32,
    angle_input: String,
    velocity_input: String,
    t: f32,
    impact: bool,
    on_screen: bool,
    player_hit: Option<usize>,
    x: f32,
    y: f32,
}

impl ShotState {
    fn new() -> Self {
        ShotState {
            phase: ShotPhase::InputAngle,
            angle: 0.0,
            velocity: 0.0,
            angle_input: String::new(),
            velocity_input: String::new(),
            t: 0.0,
            impact: false,
            on_screen: true,
            player_hit: None,
            x: 0.0,
            y: 0.0,
        }
    }
}

fn fn_ran(x: i32) -> i32 {
    gen_range(1, x + 1)
}

fn window_conf() -> Conf {
    Conf {
        window_title: "QBasic Gorillas".to_string(),
        fullscreen: true,
        ..Default::default()
    }
}

fn get_scale() -> (f32, f32, f32) {
    let sw = screen_width();
    let sh = screen_height();
    let scale = (sw / VIRTUAL_WIDTH).min(sh / VIRTUAL_HEIGHT);
    let offset_x = (sw - VIRTUAL_WIDTH * scale) / 2.0;
    let offset_y = (sh - VIRTUAL_HEIGHT * scale) / 2.0;
    (scale, offset_x, offset_y)
}

fn draw_scaled_rect(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let (scale, ox, oy) = get_scale();
    draw_rectangle(ox + x * scale, oy + y * scale, w * scale, h * scale, color);
}

fn draw_scaled_circle(x: f32, y: f32, r: f32, color: Color) {
    let (scale, ox, oy) = get_scale();
    draw_circle(ox + x * scale, oy + y * scale, r * scale, color);
}

fn draw_scaled_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    let (scale, ox, oy) = get_scale();
    draw_line(
        ox + x1 * scale, oy + y1 * scale,
        ox + x2 * scale, oy + y2 * scale,
        thickness * scale, color
    );
}

fn draw_scaled_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let (scale, ox, oy) = get_scale();
    draw_text(text, ox + x * scale, oy + y * scale, font_size * scale, color);
}

fn draw_scaled_arc(cx: f32, cy: f32, radius: f32, thickness: f32, start_angle: f32, sweep: f32, color: Color) {
    let (scale, ox, oy) = get_scale();
    let steps = 16;
    for i in 0..steps {
        let a1 = start_angle + sweep * (i as f32 / steps as f32);
        let a2 = start_angle + sweep * ((i + 1) as f32 / steps as f32);
        let x1 = ox + (cx + radius * a1.cos()) * scale;
        let y1 = oy + (cy + radius * a1.sin()) * scale;
        let x2 = ox + (cx + radius * a2.cos()) * scale;
        let y2 = oy + (cy + radius * a2.sin()) * scale;
        draw_line(x1, y1, x2, y2, thickness * scale, color);
    }
}

fn center_text(row: f32, text: &str) {
    let (scale, _, _) = get_scale();
    let font_size = 20.0;
    let text_width = measure_text(text, None, (font_size * scale) as u16, 1.0).width;
    let (_, ox, oy) = get_scale();
    let x = ox + (VIRTUAL_WIDTH * scale - text_width) / 2.0;
    let y = oy + row * 14.0 * scale;
    draw_text(text, x, y, font_size * scale, WHITE);
}

fn draw_sun(mouth_shocked: bool) {
    let x = VIRTUAL_WIDTH / 2.0;
    let y = 25.0;

    draw_scaled_rect(x - 22.0, y - 18.0, 44.0, 36.0, BACK_COLOR);
    draw_scaled_circle(x, y, 12.0, SUN_ATTR);

    draw_scaled_line(x - 20.0, y, x + 20.0, y, 2.0, SUN_ATTR);
    draw_scaled_line(x, y - 15.0, x, y + 15.0, 2.0, SUN_ATTR);
    draw_scaled_line(x - 15.0, y - 10.0, x + 15.0, y + 10.0, 2.0, SUN_ATTR);
    draw_scaled_line(x - 15.0, y + 10.0, x + 15.0, y - 10.0, 2.0, SUN_ATTR);
    draw_scaled_line(x - 8.0, y - 13.0, x + 8.0, y + 13.0, 2.0, SUN_ATTR);
    draw_scaled_line(x - 8.0, y + 13.0, x + 8.0, y - 13.0, 2.0, SUN_ATTR);
    draw_scaled_line(x - 18.0, y - 5.0, x + 18.0, y + 5.0, 2.0, SUN_ATTR);
    draw_scaled_line(x - 18.0, y + 5.0, x + 18.0, y - 5.0, 2.0, SUN_ATTR);

    if mouth_shocked {
        draw_scaled_circle(x, y + 5.0, 3.0, BLACK);
    } else {
        // QBasic CIRCLE arc from 210 to 330 degrees draws a smile
        // In screen coords (Y down), we need to draw from 30 to 150 degrees
        let start_angle = 30.0 * PI / 180.0;
        let end_angle = 150.0 * PI / 180.0;
        let steps = 20;
        for i in 0..steps {
            let a1 = start_angle + (end_angle - start_angle) * (i as f32 / steps as f32);
            let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / steps as f32);
            let x1 = x + 8.0 * a1.cos();
            let y1 = y + 8.0 * a1.sin();
            let x2 = x + 8.0 * a2.cos();
            let y2 = y + 8.0 * a2.sin();
            draw_scaled_line(x1, y1, x2, y2, 1.0, BLACK);
        }
    }

    draw_scaled_circle(x - 3.0, y - 2.0, 1.5, BLACK);
    draw_scaled_circle(x + 3.0, y - 2.0, 1.5, BLACK);
}

fn draw_gorilla(x: f32, y: f32, arms: i32) {
    draw_scaled_rect(x - 4.0, y, 7.0, 6.0, OBJECT_COLOR);
    draw_scaled_rect(x - 5.0, y + 2.0, 9.0, 2.0, OBJECT_COLOR);
    draw_scaled_line(x - 3.0, y + 2.0, x + 2.0, y + 2.0, 1.0, BLACK);
    draw_scaled_circle(x - 1.5, y + 4.0, 0.8, BLACK);
    draw_scaled_circle(x + 1.5, y + 4.0, 0.8, BLACK);
    draw_scaled_line(x - 3.0, y + 7.0, x + 2.0, y + 7.0, 1.0, OBJECT_COLOR);
    draw_scaled_rect(x - 8.0, y + 8.0, 15.0, 6.0, OBJECT_COLOR);
    draw_scaled_rect(x - 6.0, y + 15.0, 11.0, 5.0, OBJECT_COLOR);

    for i in 0..5 {
        let offset = i as f32;
        draw_scaled_arc(x + offset, y + 25.0, 10.0, 1.0, 3.0 * PI / 4.0, 9.0 * PI / 8.0 - 3.0 * PI / 4.0, OBJECT_COLOR);
        draw_scaled_arc(x - 6.0 + offset, y + 25.0, 10.0, 1.0, 15.0 * PI / 8.0, 2.0 * PI + PI / 4.0 - 15.0 * PI / 8.0, OBJECT_COLOR);
    }

    draw_scaled_arc(x - 5.0, y + 10.0, 5.0, 1.0, 3.0 * PI / 2.0, 2.0 * PI - 3.0 * PI / 2.0, BLACK);
    draw_scaled_arc(x + 5.0, y + 10.0, 5.0, 1.0, PI, 3.0 * PI / 2.0 - PI, BLACK);

    for i in -5..0 {
        let offset = i as f32;
        match arms {
            1 => {
                draw_scaled_arc(x + offset, y + 14.0, 9.0, 1.0, 3.0 * PI / 4.0, 5.0 * PI / 4.0 - 3.0 * PI / 4.0, OBJECT_COLOR);
                draw_scaled_arc(x + 5.0 + offset, y + 4.0, 9.0, 1.0, 7.0 * PI / 4.0, 2.0 * PI + PI / 4.0 - 7.0 * PI / 4.0, OBJECT_COLOR);
            }
            2 => {
                draw_scaled_arc(x + offset, y + 4.0, 9.0, 1.0, 3.0 * PI / 4.0, 5.0 * PI / 4.0 - 3.0 * PI / 4.0, OBJECT_COLOR);
                draw_scaled_arc(x + 5.0 + offset, y + 14.0, 9.0, 1.0, 7.0 * PI / 4.0, 2.0 * PI + PI / 4.0 - 7.0 * PI / 4.0, OBJECT_COLOR);
            }
            _ => {
                draw_scaled_arc(x + offset, y + 14.0, 9.0, 1.0, 3.0 * PI / 4.0, 5.0 * PI / 4.0 - 3.0 * PI / 4.0, OBJECT_COLOR);
                draw_scaled_arc(x + 5.0 + offset, y + 14.0, 9.0, 1.0, 7.0 * PI / 4.0, 2.0 * PI + PI / 4.0 - 7.0 * PI / 4.0, OBJECT_COLOR);
            }
        }
    }
}

fn draw_banana(xc: f32, yc: f32, rotation: i32) {
    match rotation % 4 {
        0 => draw_scaled_arc(xc + 4.0, yc + 3.0, 4.0, 3.0, PI / 2.0, PI, YELLOW),
        1 => draw_scaled_arc(xc + 3.0, yc, 4.0, 3.0, 0.0, PI, YELLOW),
        2 => draw_scaled_arc(xc + 3.0, yc + 6.0, 4.0, 3.0, PI, PI, YELLOW),
        3 => draw_scaled_arc(xc, yc + 3.0, 4.0, 3.0, 3.0 * PI / 2.0, PI, YELLOW),
        _ => {}
    }
}

fn generate_cityscape(state: &mut GameState) {
    let mut x = 2.0;
    let slope = fn_ran(6);
    let mut new_ht: f32 = match slope {
        1 => 15.0,
        2 => 130.0,
        3..=5 => 15.0,
        _ => 130.0,
    };

    let bottom_line = 335.0;
    let ht_inc = 10.0;
    let def_b_width = 37.0;
    let random_height = 120.0;
    let w_width = 3.0;
    let w_height = 6.0;
    let w_dif_v = 15.0;
    let w_dif_h = 10.0;
    let max_height = SUN_HT + 10.0;

    let mut cur_building = 1;
    state.buildings.clear();

    while x <= VIRTUAL_WIDTH - ht_inc {
        match slope {
            1 => new_ht += ht_inc,
            2 => new_ht -= ht_inc,
            3..=5 => {
                if x > VIRTUAL_WIDTH / 2.0 {
                    new_ht -= 2.0 * ht_inc;
                } else {
                    new_ht += 2.0 * ht_inc;
                }
            }
            _ => {
                if x > VIRTUAL_WIDTH / 2.0 {
                    new_ht += 2.0 * ht_inc;
                } else {
                    new_ht -= 2.0 * ht_inc;
                }
            }
        }

        let mut b_width = fn_ran(def_b_width as i32) as f32 + def_b_width;
        if x + b_width > VIRTUAL_WIDTH {
            b_width = VIRTUAL_WIDTH - x - 2.0;
        }

        let mut b_height = fn_ran(random_height as i32) as f32 + new_ht;
        if b_height < ht_inc {
            b_height = ht_inc;
        }
        if bottom_line - b_height <= max_height + G_HEIGHT {
            b_height = max_height + G_HEIGHT - 5.0;
        }

        if cur_building < state.bcoor.len() {
            state.bcoor[cur_building].x_coor = x as i32;
            state.bcoor[cur_building].y_coor = (bottom_line - b_height) as i32;
        }

        let building_color = BUILDING_COLORS[fn_ran(4) as usize - 1];

        let mut windows = Vec::new();
        let mut c = x + 3.0;
        while c < x + b_width - 3.0 {
            let mut i = b_height - 3.0;
            while i >= 7.0 {
                let win_color = if fn_ran(4) == 1 { DARKGRAY } else { WINDOW_COLOR };
                windows.push(Window {
                    x: c,
                    y: bottom_line - i,
                    width: w_width,
                    height: w_height,
                    color: win_color,
                });
                i -= w_dif_v;
            }
            c += w_dif_h;
        }

        state.buildings.push(Building {
            x,
            y: bottom_line - b_height,
            width: b_width,
            height: b_height,
            color: building_color,
            windows,
        });

        x += b_width + 2.0;
        cur_building += 1;
    }

    state.last_building = cur_building - 1;

    state.wind = fn_ran(10) - 5;
    if fn_ran(3) == 1 {
        if state.wind > 0 {
            state.wind += fn_ran(10);
        } else {
            state.wind -= fn_ran(10);
        }
    }

    state.cityscape_generated = true;
}

fn draw_cityscape(state: &GameState) {
    for building in &state.buildings {
        draw_scaled_rect(building.x - 1.0, building.y - 1.0, building.width + 2.0, building.height + 2.0, BACK_COLOR);
        draw_scaled_rect(building.x, building.y, building.width, building.height, building.color);
        for window in &building.windows {
            draw_scaled_rect(window.x, window.y, window.width, window.height, window.color);
        }
    }

    if state.wind != 0 {
        let wind_line = (state.wind * 3 * 2) as f32;
        draw_scaled_line(
            VIRTUAL_WIDTH / 2.0, VIRTUAL_HEIGHT - 5.0,
            VIRTUAL_WIDTH / 2.0 + wind_line, VIRTUAL_HEIGHT - 5.0,
            2.0, EXPLOSION_COLOR
        );
        let arrow_dir: f32 = if state.wind > 0 { -4.0 } else { 4.0 };
        draw_scaled_line(
            VIRTUAL_WIDTH / 2.0 + wind_line, VIRTUAL_HEIGHT - 5.0,
            VIRTUAL_WIDTH / 2.0 + wind_line + arrow_dir, VIRTUAL_HEIGHT - 7.0,
            2.0, EXPLOSION_COLOR
        );
        draw_scaled_line(
            VIRTUAL_WIDTH / 2.0 + wind_line, VIRTUAL_HEIGHT - 5.0,
            VIRTUAL_WIDTH / 2.0 + wind_line + arrow_dir, VIRTUAL_HEIGHT - 3.0,
            2.0, EXPLOSION_COLOR
        );
    }
}

fn place_gorillas(state: &mut GameState) {
    let x_adj = 14.0;
    let y_adj = 30.0;

    for i in 0..2 {
        let b_num = if i == 0 {
            fn_ran(2) as usize + 1
        } else {
            state.last_building - fn_ran(2) as usize
        };

        let b_width = if b_num + 1 < state.bcoor.len() {
            (state.bcoor[b_num + 1].x_coor - state.bcoor[b_num].x_coor) as f32
        } else {
            50.0
        };

        state.gorilla_x[i] = state.bcoor[b_num].x_coor as f32 + b_width / 2.0 - x_adj;
        state.gorilla_y[i] = state.bcoor[b_num].y_coor as f32 - y_adj;
    }
}

fn do_explosion(x: f32, y: f32) {
    let radius = VIRTUAL_HEIGHT / 50.0;
    let mut c = 0.0;
    while c <= radius {
        draw_scaled_circle(x, y, c, EXPLOSION_COLOR);
        c += 0.5;
    }
}

fn draw_intro_screen(sparkle_offset: i32) {
    clear_background(BLACK);

    let sparkle_chars = "*    ";
    for i in 0..80 {
        let idx = ((i + sparkle_offset) % 5) as usize;
        if sparkle_chars.chars().nth(idx) == Some('*') {
            draw_scaled_text("*", i as f32 * 8.0, 14.0, 20.0, RED);
            draw_scaled_text("*", i as f32 * 8.0, 310.0, 20.0, RED);
        }
    }

    for i in 1..21 {
        let idx = ((i + sparkle_offset) % 5) as usize;
        if sparkle_chars.chars().nth(idx) == Some('*') {
            draw_scaled_text("*", 0.0, (i * 14 + 14) as f32, 20.0, RED);
            draw_scaled_text("*", 632.0, (i * 14 + 14) as f32, 20.0, RED);
        }
    }

    draw_scaled_text("Q B a s i c    G O R I L L A S", 160.0, 56.0, 24.0, WHITE);
    draw_scaled_text("Copyright (C) IBM Corporation 1991", 180.0, 84.0, 18.0, GRAY);
    draw_scaled_text("Your mission is to hit your opponent with the exploding", 100.0, 112.0, 18.0, GRAY);
    draw_scaled_text("banana by varying the angle and power of your throw, taking", 90.0, 126.0, 18.0, GRAY);
    draw_scaled_text("into account wind speed, gravity, and the city skyline.", 100.0, 140.0, 18.0, GRAY);
    draw_scaled_text("The wind speed is shown by a directional arrow at the bottom", 85.0, 154.0, 18.0, GRAY);
    draw_scaled_text("of the playing field, its length relative to its strength.", 95.0, 168.0, 18.0, GRAY);
    draw_scaled_text("Press any key to continue", 220.0, 336.0, 18.0, GRAY);
}

fn draw_input_screen(
    current_field: InputField,
    input_buffer: &str,
    player1_input: &str,
    player2_input: &str,
    games_input: &str,
    gravity_input: &str,
) {
    clear_background(BLACK);

    let cursor = if (get_time() * 2.0) as i32 % 2 == 0 { "_" } else { " " };

    draw_scaled_text("Name of Player 1 (Default = 'Player 1'): ", 100.0, 112.0, 18.0, GRAY);
    let p1_display = if current_field == InputField::Player1Name {
        format!("{}{}", input_buffer, cursor)
    } else {
        player1_input.to_string()
    };
    draw_scaled_text(&p1_display, 430.0, 112.0, 18.0, WHITE);

    draw_scaled_text("Name of Player 2 (Default = 'Player 2'): ", 100.0, 140.0, 18.0, GRAY);
    let p2_display = if current_field == InputField::Player2Name {
        format!("{}{}", input_buffer, cursor)
    } else {
        player2_input.to_string()
    };
    draw_scaled_text(&p2_display, 430.0, 140.0, 18.0, WHITE);

    draw_scaled_text("Play to how many total points (Default = 3): ", 80.0, 168.0, 18.0, GRAY);
    let games_display = if current_field == InputField::NumGames {
        format!("{}{}", input_buffer, cursor)
    } else {
        games_input.to_string()
    };
    draw_scaled_text(&games_display, 450.0, 168.0, 18.0, WHITE);

    draw_scaled_text("Gravity in Meters/Sec (Earth = 9.8): ", 110.0, 196.0, 18.0, GRAY);
    let gravity_display = if current_field == InputField::Gravity {
        format!("{}{}", input_buffer, cursor)
    } else {
        gravity_input.to_string()
    };
    draw_scaled_text(&gravity_display, 430.0, 196.0, 18.0, WHITE);
}

fn draw_gorilla_intro_screen() {
    clear_background(BLACK);
    draw_scaled_text("--------------", 220.0, 224.0, 18.0, GRAY);
    draw_scaled_text("V = View Intro", 220.0, 252.0, 18.0, GRAY);
    draw_scaled_text("P = Play Game", 220.0, 280.0, 18.0, GRAY);
    draw_scaled_text("Your Choice?", 230.0, 308.0, 18.0, GRAY);
}

fn draw_shot_input(shot_state: &ShotState, player_num: usize) {
    let locate_col = if player_num == 0 { 10.0 } else { 500.0 };
    let cursor = if (get_time() * 2.0) as i32 % 2 == 0 { "_" } else { " " };

    draw_scaled_text("Angle:", locate_col, 28.0, 18.0, WHITE);
    let angle_display = if shot_state.phase == ShotPhase::InputAngle {
        format!("{}{}", shot_state.angle_input, cursor)
    } else {
        format!("{}", shot_state.angle as i32)
    };
    draw_scaled_text(&angle_display, locate_col + 60.0, 28.0, 18.0, WHITE);

    if shot_state.phase != ShotPhase::InputAngle {
        draw_scaled_text("Velocity:", locate_col, 42.0, 18.0, WHITE);
        let velocity_display = if shot_state.phase == ShotPhase::InputVelocity {
            format!("{}{}", shot_state.velocity_input, cursor)
        } else {
            format!("{}", shot_state.velocity as i32)
        };
        draw_scaled_text(&velocity_display, locate_col + 80.0, 42.0, 18.0, WHITE);
    }
}

fn plot_shot(state: &mut GameState, shot_state: &mut ShotState, player_num: usize, start_x: f32, start_y: f32) -> bool {
    let angle_rad = shot_state.angle / 180.0 * PI;
    let init_x_vel = angle_rad.cos() * shot_state.velocity;
    let init_y_vel = angle_rad.sin() * shot_state.velocity;

    let adjust = 4.0;
    let start_x_pos = if player_num == 1 { start_x + 25.0 } else { start_x };
    let start_y_pos = start_y - adjust - 3.0;

    shot_state.x = start_x_pos + (init_x_vel * shot_state.t) + (0.5 * (state.wind as f32 / 5.0) * shot_state.t * shot_state.t);
    shot_state.y = start_y_pos + ((-1.0 * (init_y_vel * shot_state.t)) + (0.5 * state.gravity * shot_state.t * shot_state.t)) * (VIRTUAL_HEIGHT / 350.0);

    if shot_state.x >= VIRTUAL_WIDTH - 10.0 || shot_state.x <= 3.0 || shot_state.y >= VIRTUAL_HEIGHT - 3.0 {
        shot_state.on_screen = false;
        return true;
    }

    if shot_state.y > 0.0 && shot_state.on_screen {
        for i in 0..2 {
            let gx = state.gorilla_x[i];
            let gy = state.gorilla_y[i];
            if shot_state.x >= gx - 5.0 && shot_state.x <= gx + 25.0 &&
               shot_state.y >= gy - 5.0 && shot_state.y <= gy + 30.0 {
                shot_state.impact = true;
                shot_state.player_hit = Some(i);
                return true;
            }
        }

        for i in 1..=state.last_building {
            let bx = state.bcoor[i].x_coor as f32;
            let by = state.bcoor[i].y_coor as f32;
            let next_bx = if i + 1 < state.bcoor.len() {
                state.bcoor[i + 1].x_coor as f32
            } else {
                VIRTUAL_WIDTH
            };
            if shot_state.x >= bx && shot_state.x <= next_bx && shot_state.y >= by {
                shot_state.impact = true;
                return true;
            }
        }

        let sun_x = VIRTUAL_WIDTH / 2.0;
        if (shot_state.x - sun_x).abs() < 20.0 && shot_state.y < SUN_HT {
            state.sun_hit = true;
        }

        if !shot_state.impact {
            let rot = ((shot_state.t * 10.0) as i32) % 4;
            draw_banana(shot_state.x, shot_state.y, rot);
        }
    }

    shot_state.t += 0.1;
    false
}

fn draw_game_over(state: &GameState, sparkle_offset: i32) {
    clear_background(BLACK);

    let sparkle_chars = "*    ";
    for i in 0..80 {
        let idx = ((i + sparkle_offset) % 5) as usize;
        if sparkle_chars.chars().nth(idx) == Some('*') {
            draw_scaled_text("*", i as f32 * 8.0, 14.0, 20.0, RED);
            draw_scaled_text("*", i as f32 * 8.0, 310.0, 20.0, RED);
        }
    }

    for i in 1..21 {
        let idx = ((i + sparkle_offset) % 5) as usize;
        if sparkle_chars.chars().nth(idx) == Some('*') {
            draw_scaled_text("*", 0.0, (i * 14 + 14) as f32, 20.0, RED);
            draw_scaled_text("*", 632.0, (i * 14 + 14) as f32, 20.0, RED);
        }
    }

    draw_scaled_text("GAME OVER!", 270.0, 112.0, 24.0, WHITE);
    draw_scaled_text("Score:", 290.0, 140.0, 20.0, WHITE);
    draw_scaled_text(&state.player1_name, 200.0, 168.0, 18.0, WHITE);
    draw_scaled_text(&format!("{}", state.total_wins[0]), 400.0, 168.0, 18.0, WHITE);
    draw_scaled_text(&state.player2_name, 200.0, 196.0, 18.0, WHITE);
    draw_scaled_text(&format!("{}", state.total_wins[1]), 400.0, 196.0, 18.0, WHITE);
    draw_scaled_text("Press any key to continue", 220.0, 336.0, 18.0, GRAY);
}

fn draw_play_again() {
    clear_background(BLACK);
    draw_scaled_text("Would you like to play again?", 180.0, 168.0, 24.0, MAGENTA);
    draw_scaled_text("(Y/N)", 290.0, 210.0, 20.0, WHITE);
}

fn victory_dance(state: &GameState, player: usize, frame: i32) {
    let arms = if frame % 2 == 0 { LEFT_UP } else { RIGHT_UP };
    draw_gorilla(state.gorilla_x[player], state.gorilla_y[player], arms);
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::new();
    let mut phase = GamePhase::Intro;
    let mut sparkle_offset = 0;
    let mut last_sparkle_time = 0.0;

    let mut current_input_field = InputField::Player1Name;
    let mut input_buffer = String::new();
    let mut player1_input = String::new();
    let mut player2_input = String::new();
    let mut games_input = String::new();
    let mut gravity_input = String::new();

    let mut current_player = 0;
    let mut shot_state = ShotState::new();
    let mut current_game = 0;
    let mut victory_frame = 0;
    let mut victory_timer = 0.0;
    let mut showing_victory = false;
    let mut winning_player = 0;
    let mut showing_view_intro = false;
    let mut intro_dance_frame = 0;
    let mut intro_dance_timer = 0.0;
    let mut fullscreen = true;

    loop {
        let current_time = get_time();
        if current_time - last_sparkle_time > 0.1 {
            sparkle_offset = (sparkle_offset + 1) % 5;
            last_sparkle_time = current_time;
        }

        // ESC to quit
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Cmd+Enter (Mac) or Ctrl+Enter to toggle fullscreen
        if is_key_pressed(KeyCode::Enter) && (is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper) || is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }

        match phase {
            GamePhase::Intro => {
                draw_intro_screen(sparkle_offset);

                if get_last_key_pressed().is_some() || is_mouse_button_pressed(MouseButton::Left) {
                    phase = GamePhase::GetInputs;
                    input_buffer.clear();
                }
            }

            GamePhase::GetInputs => {
                draw_input_screen(
                    current_input_field,
                    &input_buffer,
                    &player1_input,
                    &player2_input,
                    &games_input,
                    &gravity_input,
                );

                if let Some(key) = get_last_key_pressed() {
                    match key {
                        KeyCode::Enter => {
                            match current_input_field {
                                InputField::Player1Name => {
                                    player1_input = if input_buffer.is_empty() {
                                        "Player 1".to_string()
                                    } else {
                                        input_buffer.chars().take(10).collect()
                                    };
                                    input_buffer.clear();
                                    current_input_field = InputField::Player2Name;
                                }
                                InputField::Player2Name => {
                                    player2_input = if input_buffer.is_empty() {
                                        "Player 2".to_string()
                                    } else {
                                        input_buffer.chars().take(10).collect()
                                    };
                                    input_buffer.clear();
                                    current_input_field = InputField::NumGames;
                                }
                                InputField::NumGames => {
                                    games_input = input_buffer.clone();
                                    let num: i32 = input_buffer.parse().unwrap_or(3);
                                    state.num_games = if num > 0 { num } else { 3 };
                                    input_buffer.clear();
                                    current_input_field = InputField::Gravity;
                                }
                                InputField::Gravity => {
                                    gravity_input = input_buffer.clone();
                                    let grav: f32 = input_buffer.parse().unwrap_or(9.8);
                                    state.gravity = if grav > 0.0 { grav } else { 9.8 };
                                    state.player1_name = player1_input.clone();
                                    state.player2_name = player2_input.clone();
                                    phase = GamePhase::GorillaIntro;
                                    current_input_field = InputField::ViewOrPlay;
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Backspace => { input_buffer.pop(); }
                        _ => {}
                    }
                }

                while let Some(c) = get_char_pressed() {
                    if c.is_alphanumeric() || c == '.' || c == ' ' {
                        if input_buffer.len() < 20 {
                            input_buffer.push(c);
                        }
                    }
                }
            }

            GamePhase::GorillaIntro => {
                if !showing_view_intro {
                    draw_gorilla_intro_screen();

                    if let Some(key) = get_last_key_pressed() {
                        match key {
                            KeyCode::V => {
                                showing_view_intro = true;
                                intro_dance_timer = get_time();
                            }
                            KeyCode::P => {
                                current_game = 0;
                                state.total_wins = [0, 0];
                                phase = GamePhase::Playing;
                                current_player = 0;
                                shot_state = ShotState::new();
                                state.reset_cityscape();
                                generate_cityscape(&mut state);
                                place_gorillas(&mut state);
                            }
                            _ => {}
                        }
                    }
                } else {
                    clear_background(BACK_COLOR);
                    center_text(2.0, "Q B A S I C   G O R I L L A S");
                    center_text(5.0, "STARRING:");
                    let p_str = format!("{} AND {}", state.player1_name, state.player2_name);
                    center_text(7.0, &p_str);

                    let x = 278.0;
                    let y = 175.0;
                    let arm_state = if intro_dance_frame % 2 == 0 { LEFT_UP } else { RIGHT_UP };
                    draw_gorilla(x - 13.0, y, arm_state);
                    draw_gorilla(x + 47.0, y, if arm_state == LEFT_UP { RIGHT_UP } else { LEFT_UP });

                    if get_time() - intro_dance_timer > 0.3 {
                        intro_dance_frame += 1;
                        intro_dance_timer = get_time();
                    }

                    if intro_dance_frame > 16 {
                        showing_view_intro = false;
                        intro_dance_frame = 0;
                        current_game = 0;
                        state.total_wins = [0, 0];
                        phase = GamePhase::Playing;
                        current_player = 0;
                        shot_state = ShotState::new();
                        state.reset_cityscape();
                        generate_cityscape(&mut state);
                        place_gorillas(&mut state);
                    }
                }
            }

            GamePhase::Playing => {
                clear_background(BACK_COLOR);

                if !state.cityscape_generated {
                    generate_cityscape(&mut state);
                    place_gorillas(&mut state);
                }

                draw_cityscape(&state);

                if state.sun_hit {
                    draw_sun(SUN_SHOCK);
                } else {
                    draw_sun(SUN_HAPPY);
                }

                for i in 0..2 {
                    if !showing_victory || i != 1 - winning_player {
                        if showing_victory && i == winning_player {
                            victory_dance(&state, winning_player, victory_frame);
                        } else {
                            draw_gorilla(state.gorilla_x[i], state.gorilla_y[i], ARMS_DOWN);
                        }
                    }
                }

                draw_scaled_text(&state.player1_name, 10.0, 14.0, 18.0, WHITE);
                let (scale, _, _) = get_scale();
                let p2_width = measure_text(&state.player2_name, None, (18.0 * scale) as u16, 1.0).width / scale;
                draw_scaled_text(&state.player2_name, VIRTUAL_WIDTH - p2_width - 10.0, 14.0, 18.0, WHITE);

                let score_text = format!("{}>Score<{}", state.total_wins[0], state.total_wins[1]);
                let score_width = measure_text(&score_text, None, (18.0 * scale) as u16, 1.0).width / scale;
                draw_scaled_text(&score_text, (VIRTUAL_WIDTH - score_width) / 2.0, 330.0, 18.0, WHITE);

                if showing_victory {
                    if get_time() - victory_timer > 0.2 {
                        victory_frame += 1;
                        victory_timer = get_time();
                    }

                    if victory_frame > 8 {
                        showing_victory = false;
                        current_game += 1;

                        if current_game >= state.num_games {
                            phase = GamePhase::GameOver;
                        } else {
                            current_player = 0;
                            shot_state = ShotState::new();
                            state.sun_hit = false;
                            state.reset_cityscape();
                            generate_cityscape(&mut state);
                            place_gorillas(&mut state);
                        }
                    }
                } else {
                    match shot_state.phase {
                        ShotPhase::InputAngle => {
                            draw_shot_input(&shot_state, current_player);

                            if let Some(key) = get_last_key_pressed() {
                                match key {
                                    KeyCode::Enter => {
                                        shot_state.angle = shot_state.angle_input.parse().unwrap_or(45.0);
                                        if current_player == 1 {
                                            shot_state.angle = 180.0 - shot_state.angle;
                                        }
                                        shot_state.phase = ShotPhase::InputVelocity;
                                    }
                                    KeyCode::Backspace => { shot_state.angle_input.pop(); }
                                    _ => {}
                                }
                            }

                            while let Some(c) = get_char_pressed() {
                                if c.is_ascii_digit() || c == '.' {
                                    shot_state.angle_input.push(c);
                                }
                            }
                        }
                        ShotPhase::InputVelocity => {
                            draw_shot_input(&shot_state, current_player);

                            if let Some(key) = get_last_key_pressed() {
                                match key {
                                    KeyCode::Enter => {
                                        shot_state.velocity = shot_state.velocity_input.parse().unwrap_or(50.0);
                                        shot_state.phase = ShotPhase::Animating;
                                        shot_state.t = 0.0;
                                        shot_state.impact = false;
                                        shot_state.on_screen = true;
                                        shot_state.player_hit = None;
                                        state.sun_hit = false;
                                    }
                                    KeyCode::Backspace => { shot_state.velocity_input.pop(); }
                                    _ => {}
                                }
                            }

                            while let Some(c) = get_char_pressed() {
                                if c.is_ascii_digit() || c == '.' {
                                    shot_state.velocity_input.push(c);
                                }
                            }
                        }
                        ShotPhase::Animating => {
                            let arms = if current_player == 0 { LEFT_UP } else { RIGHT_UP };
                            let gx = state.gorilla_x[current_player];
                            let gy = state.gorilla_y[current_player];
                            draw_gorilla(gx, gy, arms);

                            let done = plot_shot(&mut state, &mut shot_state, current_player, gx, gy);

                            if done {
                                shot_state.phase = ShotPhase::Done;
                            }
                        }
                        ShotPhase::Done => {
                            if let Some(hit_player) = shot_state.player_hit {
                                do_explosion(shot_state.x, shot_state.y);

                                if hit_player == current_player {
                                    state.total_wins[1 - current_player] += 1;
                                    winning_player = 1 - current_player;
                                } else {
                                    state.total_wins[current_player] += 1;
                                    winning_player = current_player;
                                }

                                showing_victory = true;
                                victory_frame = 0;
                                victory_timer = get_time();
                            } else if shot_state.impact {
                                do_explosion(shot_state.x + 4.0, shot_state.y + 4.0);
                                current_player = 1 - current_player;
                                shot_state = ShotState::new();
                            } else {
                                current_player = 1 - current_player;
                                shot_state = ShotState::new();
                            }

                            if !showing_victory && state.sun_hit {
                                state.sun_hit = false;
                            }
                        }
                    }
                }
            }

            GamePhase::GameOver => {
                draw_game_over(&state, sparkle_offset);

                if get_last_key_pressed().is_some() || is_mouse_button_pressed(MouseButton::Left) {
                    phase = GamePhase::PlayAgain;
                }
            }

            GamePhase::PlayAgain => {
                draw_play_again();

                if let Some(key) = get_last_key_pressed() {
                    match key {
                        KeyCode::Y => {
                            phase = GamePhase::GetInputs;
                            input_buffer.clear();
                            player1_input.clear();
                            player2_input.clear();
                            games_input.clear();
                            gravity_input.clear();
                            current_input_field = InputField::Player1Name;
                            state = GameState::new();
                        }
                        KeyCode::N => { break; }
                        _ => {}
                    }
                }
            }
        }

        next_frame().await
    }
}
