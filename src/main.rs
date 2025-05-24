use core::panic;
use crossterm::{
    ExecutableCommand,
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, Stylize},
    terminal::{self, Clear, ClearType},
};
use rand::Rng;
use std::io::{self, Write};
use std::time::Duration;

enum SnakeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
struct SnakeBodyPoint {
    x: u16,
    y: u16,
}

struct Snake {
    direction: SnakeDirection,
    body: Vec<SnakeBodyPoint>,
}

impl Snake {
    fn new(cols: &u16, rows: &u16, initial_direction: SnakeDirection) -> Self {
        let x = cols / 2;
        let y = rows / 2;
        let snake_body_point = SnakeBodyPoint { x, y };
        Snake {
            direction: initial_direction,
            body: vec![snake_body_point],
        }
    }
    fn print_body(
        &mut self,
        stdout: &mut std::io::Stdout,
        food_position: Option<(u16, u16)>,
        cols: &u16,
        rows: &u16,
    ) -> io::Result<Option<(u16, u16)>> {
        let mut new_head = self.body[0].clone();

        match self.direction {
            SnakeDirection::Up => {
                if new_head.y == 0 {
                    new_head.y = *rows - 1;
                } else {
                    new_head.y -= 1;
                }
            }
            SnakeDirection::Down => {
                new_head.y = (new_head.y + 1) % *rows;
            }
            SnakeDirection::Left => {
                if new_head.x == 0 {
                    new_head.x = *cols - 1;
                } else {
                    new_head.x -= 1;
                }
            }
            SnakeDirection::Right => {
                new_head.x = (new_head.x + 1) % *cols;
            }
        }

        // Game over if the new head collides with body
        if self
            .body
            .iter()
            .any(|segment| segment.x == new_head.x && segment.y == new_head.y)
        {
            println!("\n\n\tGame Over! You hit yourself.\n\n");
            disable_game_mode(stdout)?;
            std::process::exit(0)
        }

        // Shift the body
        self.body.insert(0, new_head.clone());

        let mut grew = false;

        if let Some(fp) = food_position {
            if new_head.x == fp.0 && new_head.y == fp.1 {
                grew = true;
            }
        }

        if !grew {
            self.body.pop(); // Remove the tail unless food was eaten
        }

        // Render snake
        for i in 0..self.body.len() {
            let current = &self.body[i];
            let ch = if i == 0 {
                // Head
                match self.direction {
                    SnakeDirection::Up => '^',
                    SnakeDirection::Down => 'v',
                    SnakeDirection::Left => '<',
                    SnakeDirection::Right => '>',
                }
            } else {
                // Tail or body segment
                let prev = &self.body[i - 1];
                if current.x == prev.x {
                    '|'
                } else if current.y == prev.y {
                    '-'
                } else {
                    's'
                }
            };

            stdout
                .execute(MoveTo(current.x, current.y))?
                .execute(style::PrintStyledContent(ch.green()))?;
        }

        if grew {
            Ok(generate_food(cols, rows, &self.body))
        } else {
            Ok(food_position)
        }
    }
}

fn generate_food(cols: &u16, rows: &u16, snake_body: &Vec<SnakeBodyPoint>) -> Option<(u16, u16)> {
    let mut available_positions = Vec::new();

    for x in 0..*cols {
        for y in 0..*rows {
            if !snake_body.iter().any(|p| p.x == x && p.y == y) {
                available_positions.push((x, y));
            }
        }
    }

    if snake_body.is_empty() {
        panic!("The game ended on perfect score");
    }

    let mut rng = rand::rng();
    Some(available_positions[rng.random_range(0..available_positions.len())])
}

// TODO
///*
///  End screen, points,
///  play again,
///  Possible refactors,
///  can I update just the body instead of cleaning all?
///  */

fn main() -> io::Result<()> {
    setup_panic_hook();
    // Note: Windows implementation of this stream does not support non-UTF-8 byte sequences
    let mut stdout = io::stdout();
    enable_game_mode(&mut stdout)?;

    // Are we starting the game?
    let start_text = "Press arrows to move, or (q, Ctrl+c) to quit.";
    let mut arrow_press = false;

    let (cols, rows) = terminal::size()?;
    let mut snake: Option<Snake> = None;
    let mut food_position: Option<(u16, u16)> = None;
    let mut timer = 500;

    // Game loop
    loop {
        // Clear the whole screen
        stdout.execute(Clear(ClearType::All))?;

        // Draw to the screen
        if arrow_press == false {
            stdout
                .execute(MoveTo(0, 0))?
                .execute(style::PrintStyledContent(start_text.magenta()))?;
        } else if let Some(ref mut s) = snake {
            // Print the snake
            let new_food_pos: Option<(u16, u16)> =
                s.print_body(&mut stdout, food_position, &cols, &rows)?;

            if let Some(nfp) = new_food_pos {
                if Some(nfp) != food_position && timer > 50 {
                    timer -= 20;
                }
                food_position = Some(nfp);
            } else {
                food_position = None;
            }

            // Print the food
            if let Some(f) = food_position {
                stdout
                    .execute(MoveTo(f.0, f.1))?
                    .execute(style::PrintStyledContent("o".red()))?;
            } else {
                food_position = generate_food(&cols, &rows, &s.body);
            }
        }
        stdout.flush()?;

        // Handle input
        if event::poll(Duration::from_millis(timer))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Left => {
                        if arrow_press == false {
                            arrow_press = true;
                            snake = Some(Snake::new(&cols, &rows, SnakeDirection::Left));
                        }
                        if let Some(ref mut s) = snake {
                            if !matches!(s.direction, SnakeDirection::Right) {
                                s.direction = SnakeDirection::Left;
                            }
                        }
                    }
                    KeyCode::Right => {
                        if arrow_press == false {
                            arrow_press = true;
                            snake = Some(Snake::new(&cols, &rows, SnakeDirection::Right));
                        }
                        if let Some(ref mut s) = snake {
                            if !matches!(s.direction, SnakeDirection::Left) {
                                s.direction = SnakeDirection::Right;
                            }
                        }
                    }
                    KeyCode::Up => {
                        if arrow_press == false {
                            arrow_press = true;
                            snake = Some(Snake::new(&cols, &rows, SnakeDirection::Up));
                        }
                        if let Some(ref mut s) = snake {
                            if !matches!(s.direction, SnakeDirection::Down) {
                                s.direction = SnakeDirection::Up;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if arrow_press == false {
                            arrow_press = true;
                            snake = Some(Snake::new(&cols, &rows, SnakeDirection::Down));
                        }
                        if let Some(ref mut s) = snake {
                            if !matches!(s.direction, SnakeDirection::Up) {
                                s.direction = SnakeDirection::Down;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_game_mode(&mut stdout)
}

// -- Enable and disable terminal functionalities for the game to work

fn enable_game_mode(stdout: &mut std::io::Stdout) -> io::Result<()> {
    // Prevents input to be forwaded to the screen but also disables Ctrl+C
    terminal::enable_raw_mode()?;
    // Hide the cursor
    stdout.execute(Hide)?;
    Ok(())
}

fn disable_game_mode(stdout: &mut std::io::Stdout) -> io::Result<()> {
    // Enable normal input again
    terminal::disable_raw_mode()?;
    // Show cursor again
    stdout.execute(Show)?;
    // Clear terminal screen
    stdout.execute(Clear(ClearType::All))?;
    Ok(println!("\n\n\t\tThe program ended.\n\n"))
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = std::io::stdout().execute(crossterm::cursor::Show);
        eprintln!("Panic: {info}");
    }));
}
