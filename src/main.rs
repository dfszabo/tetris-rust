extern crate sdl2;
mod tetris;

use rand::{Rng, SeedableRng, thread_rng};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::ttf::Sdl2TtfContext;
use std::process::exit;
use std::time::Duration;

use rayon::prelude::*;

use crate::tetris::MoveAction::*;
use sdl2::EventPump;
use tetris::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

pub fn input(mut event_pump: &mut EventPump) -> MoveAction {
    let mut action = NONE;

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => action = QUIT,
            Event::KeyDown { keycode, .. } => {
                let keycode = keycode.unwrap();
                match keycode {
                    Keycode::Escape => action = QUIT,
                    Keycode::A | Keycode::Left => action = LEFT,
                    Keycode::D | Keycode::Right => action = RIGHT,
                    Keycode::S | Keycode::Down => action = DOWN,
                    Keycode::W | Keycode::Up | Keycode::Space => action = ROTATE,
                    _ => (),
                }
            }
            _ => {}
        }
    }
    action
}

pub fn run_tetris(run_count: usize, simulate_only: bool, fitness_params: [u64; 6]) -> u32 {
    // ============
    // Initializing
    // ============
    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();
    // let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    //
    // let mut window;
    // let mut canvas;
    // let mut event_pump = sdl_context.event_pump().unwrap();
    // let mut texture_creator;
    // let mut font;
    //
    // window = video_subsystem
    //     .window("Tetris", 600, 800)
    //     .position_centered()
    //     .build()
    //     .unwrap();
    //
    // canvas = window.into_canvas().build().unwrap();
    // texture_creator = canvas.texture_creator();
    //
    // // Load font
    // font = ttf_context.load_font("NotoMono.ttf", 36).unwrap();
    // font.set_style(sdl2::ttf::FontStyle::BOLD);

    // canvas.set_draw_color(Color::RGB(255, 255, 255));
    // canvas.clear();
    // canvas.present();

    // ==========
    // Game logic
    // ==========

    let mut game = Game::new();
    let mut loop_counter = 0usize;
    let mut rng = StdRng::from_seed([0; 32]);
    let mut round_counter = 0;
    let mut score_accumulator = 0;

    'gameloop: loop {
        loop_counter += 1;
        let mut action = NONE;

        // if !simulate_only {
        //     // Handling input
        //     action = input(&mut event_pump);
        // }

        // Update
        // force down the piece in every few iteration
        if loop_counter > 20 {
            loop_counter = 0;
            // if cant, then add the piece to the board and spawn a new one
            if !game.move_piece_down() {
                game.add_current_piece();
                let kind = rng.gen::<usize>() % 7;

                if !game.does_piece_fit(kind, 0, 0, 5) {
                    round_counter += 1;
                    score_accumulator += game.score;

                    game = Game::new();

                    if round_counter >= run_count {
                        return score_accumulator / run_count as u32;
                    }
                }

                game.score += 1;
                game.find_and_remove_solved_lines();

                let new_piece = Piece {
                    kind,
                    rotation: 0,
                    x: 0,
                    y: 5,
                };

                game.curr_piece = new_piece;
            }

            // if the current input action was moving down then do not do it since the
            // piece was already moved down
            if action == DOWN {
                action = NONE;
            }
        }

        let action = game.bot(fitness_params);

        match action {
            LEFT => game.move_piece_left(),
            RIGHT => game.move_piece_right(),
            DOWN => game.move_piece_down(),
            ROTATE => game.rotate_piece(),
            QUIT => break 'gameloop,
            _ => false,
        };

        // Draw
        if simulate_only {
            continue;
        }
        //
        // game.add_current_piece();
        // canvas.set_draw_color(Color::RGB(255, 255, 255));
        // canvas.clear();
        //
        // for row in 0..HEIGHT as usize {
        //     for col in 0..WIDTH as usize {
        //         if game.board[row][col] != 0 {
        //             canvas.set_draw_color(Color::RGB(
        //                 128,
        //                 game.board[row][col] * (255 / 7),
        //                 game.board[row][col] * (255 / 7),
        //             ));
        //         } else {
        //             canvas.set_draw_color(Color::RGB(0, 0, 0));
        //         }
        //
        //         canvas.fill_rect(Rect::new(
        //             (col as u32 * RECT_DIM) as i32,
        //             (row as u32 * RECT_DIM) as i32,
        //             RECT_DIM,
        //             RECT_DIM,
        //         ));
        //     }
        // }
        // // render a surface, and convert it to a texture bound to the canvas
        // let score = format!("Score {}", game.score);
        // let surface = font
        //     .render(&score)
        //     .blended(Color::RGB(0, 0, 0))
        //     .map_err(|e| e.to_string())
        //     .unwrap();
        // let texture = texture_creator
        //     .create_texture_from_surface(&surface)
        //     .map_err(|e| e.to_string())
        //     .unwrap();
        // let target = Rect::new(SCREEN_WIDTH as i32 + 20, 0, score.len() as u32 * 15, 44);
        // canvas.copy(&texture, None, Some(target)).unwrap();
        //
        // game.remove_current_piece();
        //
        // canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 600));
    }
    0
}

const MAX_POSSIBLE_VAL: u64 = 10000;
const POP_SIZE: usize = 1000;
const RUN_AMOUNT: usize = 5;
const PARENTS_RATIO: usize = 2;
const PARENTS_SIZE: usize = POP_SIZE / PARENTS_RATIO;
const TARGET_SCORE: u64 = 100000;
const MAX_GENERATION: u64 = 1000;
const MUTATION_PROBABILITY: usize = 20;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Copy, Clone)]
struct DNA {
    params: [u64; 6],
    score: u64,
}

impl DNA {
    pub fn new(max_val: u64) -> Self {
        let mut rng = rand::thread_rng();

        DNA {
            params: [
                rng.gen::<u64>() % max_val,
                rng.gen::<u64>() % max_val,
                rng.gen::<u64>() % max_val,
                rng.gen::<u64>() % max_val,
                rng.gen::<u64>() % max_val,
                rng.gen::<u64>() % max_val,
            ],
            score: 0,
        }
    }
}

pub fn main() {
    let mut population: [DNA; POP_SIZE] = [DNA::new(MAX_POSSIBLE_VAL); POP_SIZE];
    let mut generation = 0;
    let mut parents: [DNA; PARENTS_SIZE] = [DNA::new(MAX_POSSIBLE_VAL); PARENTS_SIZE];
    let mut best = DNA {
        params: [0u64; 6],
        score: 0u64,
    };
    // Elapsed time 170428 ms
    // Runs #5, Generation #53, Current best score 5125
    //     [1497] [1605] [225] [142] [1095] [718]
    //     -----------------------------------------------
    // population[0] = DNA {
    //     params: [292u64, 481u64, 750u64, 814u64, 172u64, 168u64],
    //     score: 0u64,
    // };

    let mut rng = rand::thread_rng();

    while best.score < TARGET_SCORE && generation < MAX_GENERATION {
        generation += 1;

        use std::time::Instant;
        let start = Instant::now();

        // for idx in 0..POP_SIZE {
        //     population[idx].score = run_tetris(RUN_AMOUNT, true, population[idx].params) as u64;
        //
        //     // if population[idx].score > best.score {
        //     //     print!("[[{}]] ", population[idx].score);
        //     // } else {
        //     //     print!("{} ", population[idx].score);
        //     // }
        // }

        let mut chunked_populataion: Vec<(usize, &mut [DNA])> =
            population.chunks_mut(10).enumerate().collect();

        chunked_populataion
            .par_iter_mut()
            .for_each(|(i, pop_chunk)| {

                for idx in 0..10usize {
                    pop_chunk[idx].score =
                        run_tetris(RUN_AMOUNT, true, pop_chunk[idx].params) as u64;
                }
            });

        // get the elapsed time
        println!("Elapsed time {:} ms", start.elapsed().as_millis());

        population.sort_by(|a, b| b.score.cmp(&a.score));

        if population[0].score > best.score {
            best = population[0];
        }

        println!(
            "Runs #{}, Generation #{}, Current best score {}",
            RUN_AMOUNT, generation, best.score
        );
        for idx in best.params.iter() {
            print!("[{}] ", *idx);
        }
        println!("\n-----------------------------------------------\n");

        // choosing the parents
        for idx in 0..PARENTS_SIZE {
            parents[idx] = population[idx];
        }

        for idx in (PARENTS_SIZE - POP_SIZE / 12)..PARENTS_SIZE {
            parents[idx] = population[rng.gen::<usize>() % PARENTS_SIZE];
        }

        let mut parent1_idx = 0usize;
        let mut parent2_idx = 0usize;

        // making the next generation population
        for idx in 0..PARENTS_SIZE {
            // chose two unique parent
            parent1_idx = rng.gen::<usize>() % PARENTS_SIZE;
            parent2_idx = rng.gen::<usize>() % PARENTS_SIZE;

            while parent1_idx == parent2_idx {
                parent2_idx = rng.gen::<usize>() % PARENTS_SIZE;
            }

            // crossover
            population[idx] = parents[parent1_idx];
            for param_idx in 0..5usize {
                // 50% chance to crossover the parameter
                if rng.gen::<usize>() % 2 == 0 {
                    population[idx].params[param_idx] = parents[parent2_idx].params[param_idx];
                }
            }

            // mutation
            if rng.gen::<usize>() % MUTATION_PROBABILITY == 0 {
                let param_idx = rng.gen::<usize>() % 6;
                let new_val =
                    // do a bigger mutation in 1/3 of the cases
                    if rng.gen::<usize>() % 3 == 0 {
                        population[idx].params[param_idx] as f64 * rng.gen_range(-0.25, 0.25)
                    }
                    // do a smaller mutation otherwise
                    else {
                        population[idx].params[param_idx] as f64 * rng.gen_range(-0.05, 0.05)
                    };
                if new_val >= 0.0f64 {
                    population[idx].params[param_idx] = new_val as u64;
                }
            }

            // choose randomly the rest of the population
            for idx in PARENTS_SIZE..POP_SIZE {
                population[idx] = DNA::new(MAX_POSSIBLE_VAL);
            }
        }
    }
}
