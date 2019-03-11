extern crate sdl2;
extern crate rand;

use std::ops::{Add, Sub, Mul};
use std::io::prelude::*;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

const WINDOW_WIDTH: u32 = 641;
const WINDOW_HEIGHT: u32 = 800;
const BG_COLOR: Color = Color{r: 0, g: 0, b: 0, a: 255};

const GRID_SIZE: usize = 16;
const BLOCK_SIZE: i32 = (WINDOW_WIDTH-1) as i32/GRID_SIZE as i32;
const FONT_SIZE: u16 = 20;

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) => (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));

#[derive(Debug, Copy, Clone)]
struct Vector {
    x: f32,
    y: f32,
}
impl Vector {
    fn normalize(&mut self) {
        let ln = ((self.x*self.x + self.y*self.y) as f32).sqrt();
        if ln == 0.0 {
            return;
        }

        let div = 1.0 / ln;
        self.x *= div;
        self.y *= div;
    }

    fn dot(&self, other: Vector) -> f32 {
        self.x*other.x + self.y*other.y
    }
}
impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector{ x: self.x + other.x, y: self.y + other.y }
    }
}
impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector{ x: self.x - other.x, y: self.y - other.y }
    }
}
impl Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, other: f32) -> Vector {
        Vector{ x: other*self.x, y: other*self.y }
    }
}

#[derive(Copy, Clone)]
struct Block {
    count: usize,
    color: Color,
}
impl Block {
    fn draw(&self, canvas: &mut Canvas<Window>, x: i32, y: i32) {
        let points: [Point; 5] = [
            Point::new(x, y),
            Point::new(x+BLOCK_SIZE, y),
            Point::new(x+BLOCK_SIZE, y+BLOCK_SIZE),
            Point::new(x, y+BLOCK_SIZE),
            Point::new(x, y),
        ];

        let color = canvas.draw_color();
        canvas.set_draw_color(self.color);
        let _ = canvas.draw_lines(&points[..]);
        canvas.set_draw_color(color);
    }
}

struct Ball {
    pos: Vector,
    radius: f32,
    dir: Vector,
    speed: f32,
}
impl Ball {
    fn draw(&self, canvas: &mut Canvas<Window>) {
        let x = self.pos.x as i32;
        let y = self.pos.y as i32;
        let r = self.radius as i32;

        let points: [Point; 9] = [
            Point::new(x+r, y),
            Point::new(x+(2*r/3), y+(2*r/3)),
            Point::new(x, y+r),
            Point::new(x-(2*r/3), y+(2*r/3)),
            Point::new(x-r, y),
            Point::new(x-(2*r/3), y-(2*r/3)),
            Point::new(x, y-r),
            Point::new(x+(2*r/3), y-(2*r/3)),
            Point::new(x+r, y),
        ];

        let color = canvas.draw_color();
        canvas.set_draw_color(Color{r: 255, g: 255, b: 255, a: 255});
        let _ = canvas.draw_lines(&points[..]);
        canvas.set_draw_color(color);
    }
}

#[derive(Eq, PartialEq)]
enum BallState {
    WaitingFirstBall,
    WaitingLastBall,
}

#[derive(Eq, PartialEq)]
enum ShootState {
    WaitingToShoot,
    Shooting,
}

struct Player {
    pos: Vector,
    ball_state: BallState,
    shoot_state: ShootState,
    aim: Vector,
    ball_count: usize,
    balls_shot: usize,
    score: usize,
}
impl Player {
    fn draw(&self, canvas: &mut Canvas<Window>) {
        let x = self.pos.x as i32;
        let y = self.pos.y as i32;
        let r = 20;

        let points: [Point; 5] = [
            Point::new(x+r, y),
            Point::new(x, y+r),
            Point::new(x-r, y),
            Point::new(x, y-r),
            Point::new(x+r, y),
        ];

        let color = canvas.draw_color();
        canvas.set_draw_color(Color{r: 255, g: 0, b: 0, a: 255});
        let _ = canvas.draw_lines(&points[..]);
        canvas.set_draw_color(color);
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem.window("PCTan", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .unwrap();

    let mut font = ttf_context.load_font("roboto.ttf", FONT_SIZE).unwrap();
    font.set_style(sdl2::ttf::STYLE_NORMAL);

    let mut canvas = window.into_canvas().accelerated().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut balls: Vec<Ball> = Vec::new();
    let mut player = Player{pos: Vector{x: WINDOW_WIDTH as f32/2.0, y: WINDOW_HEIGHT as f32 - 11.0},
                            ball_state: BallState::WaitingFirstBall,
                            shoot_state: ShootState::WaitingToShoot,
                            aim: Vector{x: 0.0, y: 0.0},
                            ball_count: 1,
                            balls_shot: 0,
                            score: 0};
    let mut ball_timer = 0;
    let mut blocks: [[Block; GRID_SIZE]; GRID_SIZE] = [[Block{count: 0, color: BG_COLOR}; GRID_SIZE]; GRID_SIZE];
    let mut mouse = player.pos;
    let mut first_ball_x = player.pos.x;

    let file_score: usize;
    {
        let file = std::fs::File::open("hiscore");
        match file {
            Ok(mut f) => {
                let mut file_score_str = String::new();
                let _ = f.read_to_string(&mut file_score_str);
                file_score = file_score_str.parse::<usize>().unwrap_or(0);
            },
            Err(_) => {
                file_score = 0;
            }
        }
    }

    for i in 0..GRID_SIZE {
        blocks[1][i].count = if rand::random() { 1 } else { 0 };
        blocks[1][i].color = Color{r: 250, g: 100, b: 50, a: 255};
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseButtonUp { mouse_btn: button, x, y, .. } => {
                    match button {
                        sdl2::mouse::MouseButton::Left => {
                            if player.shoot_state == ShootState::WaitingToShoot {
                                let mut dir = Vector{x: x as f32, y: y as f32};
                                dir = dir - player.pos;
                                dir.normalize();

                                player.aim = dir;
                                player.shoot_state = ShootState::Shooting;
                                player.ball_state = BallState::WaitingFirstBall;
                                player.balls_shot = 0;
                            }
                        },

                        _ => {},
                    }
                },
                Event::MouseMotion { x, y, .. } => {
                    mouse.x = x as f32;
                    mouse.y = y as f32;
                }

                _ => {}
            }
        }

        canvas.set_draw_color(BG_COLOR);
        canvas.clear();

        if player.shoot_state == ShootState::Shooting && ball_timer >= 3 && player.balls_shot != player.ball_count {
            balls.push(Ball{pos: player.pos, radius: 10.0, dir: player.aim, speed: 10.0});

            ball_timer = 0;
            player.balls_shot += 1;
        }

        let mut removable = -1;
        for i in 0..balls.len() {
            let ball = &mut balls[i];

            ball.pos.y += ball.speed * ball.dir.y;
            ball.pos.x += ball.speed * ball.dir.x;

            if ball.pos.y - ball.radius <= 0.0 {
                ball.dir.y = -ball.dir.y;
            }
            if ball.pos.x - ball.radius <= 0.0 || ball.pos.x + ball.radius >= WINDOW_WIDTH as f32 {
                ball.dir.x = -ball.dir.x;
            }
            if ball.pos.y + ball.radius >= WINDOW_HEIGHT as f32 {
                if player.ball_state == BallState::WaitingFirstBall {
                    first_ball_x = ball.pos.x;
                    player.ball_state = BallState::WaitingLastBall;
                }
                removable = i as isize;
            }

            for i in 0..GRID_SIZE {
                for j in 0..GRID_SIZE {
                    let block = &mut blocks[i][j];

                    if block.count > 0 {
                        let x = (j as i32*BLOCK_SIZE) as f32;
                        let y = (i as i32*BLOCK_SIZE) as f32;

                        let dx = ball.pos.x - x.max(ball.pos.x.min(x + BLOCK_SIZE as f32));
                        let dy = ball.pos.y - y.max(ball.pos.y.min(y + BLOCK_SIZE as f32));
                        if (dx * dx + dy * dy) < (ball.radius * ball.radius) {
                            let mut dir = Vector{ x: dx, y: dy };
                            dir.normalize();

                            ball.dir = ball.dir + dir*2.0;
                            ball.dir.normalize();

                            block.count -= 1;

                            if block.count == 0 {
                                player.score += 1;
                            }
                        }
                    }
                }
            }

            ball.draw(&mut canvas);
        }

        if removable != -1 {
            balls.swap_remove(removable as usize);
        }

        if balls.is_empty() && player.ball_state == BallState::WaitingLastBall {
            player.pos.x = first_ball_x;
            player.ball_count += 1;
            player.shoot_state = ShootState::WaitingToShoot;
            player.ball_state = BallState::WaitingFirstBall;

            if blocks[GRID_SIZE-1].iter().filter(|x| x.count > 0).count() > 0 {
                println!["Score: {}", player.score];
                println!["Game Over"];

                let total_score = if player.score > file_score { player.score } else { file_score };

                let mut file = std::fs::File::create("hiscore").unwrap();
                let _ = file.write(&total_score.to_string().into_bytes());

                break 'running;
            }

            for i in (1..GRID_SIZE).rev() {
                let (top, bottom) = blocks.split_at_mut(i);
                let upper_row = &top[i-1];
                let bottom_row = &mut bottom[0];

                bottom_row.copy_from_slice(upper_row);
            }

            for i in 0..GRID_SIZE {
                blocks[1][i].count = if rand::random() { player.ball_count } else { 0 };
                blocks[1][i].color = Color{r: 250, g: 100, b: 50, a: 255};
            }
        }

        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                let block = &blocks[i][j];

                if block.count > 0 {
                    let surface = font.render(&block.count.to_string()).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
                    let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
                    let texture_info = texture.query();

                    let x = j as i32*BLOCK_SIZE + BLOCK_SIZE/2 - texture_info.width as i32/2;
                    let y = i as i32*BLOCK_SIZE + BLOCK_SIZE/2 - texture_info.height as i32/2;
                    canvas.copy(&texture, None, Some(rect![x, y, texture_info.width, texture_info.height])).unwrap();

                    block.draw(&mut canvas, j as i32*BLOCK_SIZE, i as i32*BLOCK_SIZE);
                }
            }
        }

        {
            let score = format!["Score: {}", player.score.to_string()];
            let surface = font.render(&score).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
            let texture_info = texture.query();

            canvas.copy(&texture, None, Some(rect![10, 10, texture_info.width, texture_info.height])).unwrap();
        }
        {
            let hi_score = format!["HiScore: {}", file_score.to_string()];
            let surface = font.render(&hi_score).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
            let texture_info = texture.query();

            canvas.copy(&texture, None, Some(rect![WINDOW_WIDTH - texture_info.width - 10, 10, texture_info.width, texture_info.height])).unwrap();
        }

        player.draw(&mut canvas);

        if player.shoot_state == ShootState::WaitingToShoot {
            let color = canvas.draw_color();
            canvas.set_draw_color(Color{r: 255, g: 255, b: 255, a: 255});
            let _ = canvas.draw_line(Point::new(player.pos.x as i32, player.pos.y as i32), Point::new(mouse.x as i32, mouse.y as i32));
            canvas.set_draw_color(color);
        }
        let color = canvas.draw_color();
        canvas.set_draw_color(Color{r: 0, g: 0, b: 255, a: 255});
        let _ = canvas.draw_line(Point::new(0, GRID_SIZE as i32 * BLOCK_SIZE), Point::new(GRID_SIZE as i32 * BLOCK_SIZE, GRID_SIZE as i32 * BLOCK_SIZE));
        canvas.set_draw_color(color);

        ball_timer += 1;
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
        canvas.present();
    }
}
