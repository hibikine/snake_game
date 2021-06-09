extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::{
    keyboard::Key, Button, ButtonEvent, ButtonState, RenderArgs, RenderEvent, UpdateArgs,
    UpdateEvent,
};
use piston::window::WindowSettings;
use rand::prelude::*;
use std::collections::LinkedList;
use std::iter::FromIterator;

#[derive(Clone, PartialEq)]
enum Direction {
    Right,
    Left,
    Up,
    Down,
}
#[derive(PartialEq, Debug)]
enum SnakeState {
    Dead,
    Live,
}

struct Snake {
    body: LinkedList<(i32, i32)>,
    dir: Direction,
    state: SnakeState,
}
const CELL_SIZE: u32 = 20;
impl Snake {
    fn new() -> Self {
        Snake {
            body: LinkedList::from_iter((vec![(0, 0), (0, 1)]).into_iter()),
            dir: Direction::Right,
            state: SnakeState::Live,
        }
    }
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        let squares: Vec<graphics::types::Rectangle> = self
            .body
            .iter()
            .map(|&(x, y)| {
                graphics::rectangle::square(
                    (x * CELL_SIZE as i32) as f64,
                    (y * CELL_SIZE as i32) as f64,
                    CELL_SIZE as f64,
                )
            })
            .collect();
        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            squares
                .into_iter()
                .for_each(|square| graphics::rectangle(RED, square, transform, gl));
        })
    }
    fn hit_self(&self, new_head: &(i32, i32)) -> SnakeState {
        match self.body.iter().all(|b| b != new_head) {
            true => SnakeState::Live,
            false => SnakeState::Dead,
        }
    }
    fn update(&mut self, apple: &mut Apple) -> SnakeState {
        let mut new_head = (*self.body.front().expect("Snake has no body")).clone();
        match self.dir {
            Direction::Left => {
                new_head.0 += BOARD_WIDTH as i32 - 1;
                new_head.0 %= BOARD_WIDTH as i32
            }
            Direction::Right => {
                new_head.0 += 1;
                new_head.0 %= BOARD_WIDTH as i32
            }
            Direction::Up => {
                new_head.1 += BOARD_HEIGHT as i32 - 1;
                new_head.1 %= BOARD_HEIGHT as i32
            }
            Direction::Down => {
                new_head.1 += 1;
                new_head.1 %= BOARD_HEIGHT as i32
            }
        }
        match self.hit_self(&new_head) {
            SnakeState::Live => {
                self.body.push_front(new_head);
                if new_head == apple.pos {
                    apple.spawn_new_apple(&self);
                } else {
                    self.body.pop_back().unwrap();
                }
                SnakeState::Live
            }
            SnakeState::Dead => {
                self.state = SnakeState::Dead;
                SnakeState::Dead
            }
        }
    }
}

struct Apple {
    pos: (i32, i32),
    rng: ThreadRng,
}

impl Apple {
    fn generate_new_pos(&mut self) {
        self.pos = (
            (self.rng.next_u32() % BOARD_WIDTH) as i32,
            (self.rng.next_u32() % BOARD_HEIGHT) as i32,
        );
    }
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut apple = Apple {
            rng: rng,
            pos: (0, 0),
        };
        apple.generate_new_pos();
        apple
    }
    fn spawn_new_apple(&mut self, snake: &Snake) {
        while {
            self.generate_new_pos();
            snake.body.iter().all(|b| b == &self.pos)
        } {}
    }
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
        let square = graphics::rectangle::square(
            (self.pos.0 * CELL_SIZE as i32) as f64,
            (self.pos.1 * CELL_SIZE as i32) as f64,
            CELL_SIZE as f64,
        );
        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            graphics::rectangle(YELLOW, square, transform, gl);
        });
    }
}

struct App {
    gl: GlGraphics,
    snake: Snake,
    apple: Apple,
}

impl App {
    fn new(opengl: OpenGL) -> Self {
        App {
            gl: GlGraphics::new(opengl),
            snake: Snake::new(),
            apple: Apple::new(),
        }
    }
    fn render(&mut self, args: &RenderArgs) {
        let GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(GREEN, gl);
        });
        self.snake.render(&mut self.gl, args);
        self.apple.render(&mut self.gl, args)
    }
    fn update(&mut self) {
        if self.snake.state == SnakeState::Live {
            self.snake.update(&mut self.apple);
        }
    }
    fn reset(&mut self) {
        self.snake = Snake::new();
        self.apple = Apple::new();
    }
    fn pressed(&mut self, btn: &Button) {
        let last_direction = self.snake.dir.clone();
        self.snake.dir = match btn {
            &Button::Keyboard(Key::Up) if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down) if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left) if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right) if last_direction != Direction::Left => Direction::Right,
            _ => last_direction,
        };
        match btn {
            &Button::Keyboard(Key::R) => self.reset(),
            _ => {}
        }
    }
}

const WIDTH: u32 = 200;
const HEIGHT: u32 = 200;
const BOARD_WIDTH: u32 = WIDTH / CELL_SIZE;
const BOARD_HEIGHT: u32 = HEIGHT / CELL_SIZE;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: GlutinWindow = WindowSettings::new("Snake Game", [WIDTH, HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut app = App::new(opengl);
    let mut events = Events::new(EventSettings::new()).ups(8);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }
        if let Some(args) = e.update_args() {
            app.update();
        }
        if let Some(args) = e.button_args() {
            app.pressed(&args.button);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_update_loop() {
        let mut snake = Snake::new();
        snake.dir = Direction::Left;
        let mut apple = Apple::new();
        apple.pos = (BOARD_WIDTH as i32 - 1, BOARD_HEIGHT as i32 - 1);
        snake.update(&mut apple);
        assert_eq!(
            snake.body,
            LinkedList::from_iter((vec![(BOARD_WIDTH as i32 - 1, 0), (0, 0)]).into_iter())
        );
    }
    #[test]
    fn hit_self_dead() {
        let mut snake = Snake::new();
        snake.dir = Direction::Left;
        assert_eq!(snake.hit_self(&(0, 0)), SnakeState::Dead)
    }
    #[test]
    fn hit_self_live() {
        let mut snake = Snake::new();
        snake.dir = Direction::Left;
        assert_eq!(snake.hit_self(&(2, 0)), SnakeState::Live)
    }
    #[test]
    fn linked_list_compare() {
        assert_eq!(
            LinkedList::from_iter((vec![(-1, 0), (0, 0)]).into_iter()),
            LinkedList::from_iter((vec![(-1, 0), (0, 0)]).into_iter())
        );
    }
    #[test]
    fn test_tuple_compare() {
        let a: &(i32, i32) = &(3, 4);
        let b: &(i32, i32) = &(3, 4);
        assert_eq!(a == b, true);
    }
    #[test]
    fn test_tuple_compare_false() {
        let a: &(i32, i32) = &(3, 4);
        let b: &(i32, i32) = &(3, 6);
        assert_eq!(a == b, false);
    }
    #[test]
    fn test_mod_minus() {
        assert_eq!((-1 + 3) % 3, 2);
    }
}
