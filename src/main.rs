extern crate sdl2;
extern crate unicode_segmentation;

use std::ops::{Add, Sub};

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

const WINDOW_WIDTH: u32 = 600;
const WINDOW_HEIGHT: u32 = 900;
const BG_COLOR: Color = Color{r: 0, g: 0, b: 0, a: 255};

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) => (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
struct Block {
    count: usize,
    color: Color,
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
}
impl Player {
    fn draw(&self, canvas: &mut Canvas<Window>) {
        let x = self.pos.x as i32;
        let y = self.pos.y as i32;
        let r = 12;

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
        canvas.set_draw_color(Color{r: 255, g: 0, b: 0, a: 255});
        let _ = canvas.draw_lines(&points[..]);
        canvas.set_draw_color(color);
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("ColorCoding", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut balls: Vec<Ball> = Vec::new();
    let mut player = Player{pos: Vector{x: WINDOW_WIDTH as f32/2.0, y: WINDOW_HEIGHT as f32 - 11.0},
                            ball_state: BallState::WaitingFirstBall,
                            shoot_state: ShootState::WaitingToShoot,
                            aim: Vector{x: 0.0, y: 0.0},
                            ball_count: 1,
                            balls_shot: 0};
    let mut ball_timer = 0;
    let mut blocks: [[Block; 15]; 20] = [[Block{count: 0, color: BG_COLOR}; 15]; 20];

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

                _ => {}
            }
        }

        canvas.set_draw_color(BG_COLOR);
        canvas.clear();

        if player.shoot_state == ShootState::Shooting && ball_timer >= 7 && player.balls_shot != player.ball_count {
            balls.push(Ball{pos: player.pos, radius: 10.0, dir: player.aim, speed: 4.0});

            ball_timer = 0;
            player.balls_shot += 1;
        }

        let mut removable: Vec<usize> = Vec::new();
        let mut i = 0;
        for ball in &mut balls {
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
                    player.pos.x = ball.pos.x;
                    player.ball_state = BallState::WaitingLastBall;
                }
                removable.push(i);
            }

            ball.draw(&mut canvas);
            i += 1;
        }

        for j in removable {
            balls.swap_remove(j);
        }

        if balls.is_empty() && player.ball_state == BallState::WaitingLastBall {
            player.ball_count += 1;
            player.shoot_state = ShootState::WaitingToShoot;
            player.ball_state = BallState::WaitingFirstBall;
        }

        player.draw(&mut canvas);

        ball_timer += 1;
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
        canvas.present();
    }
}
