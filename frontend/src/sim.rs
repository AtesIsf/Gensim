use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use genalgo::*;
use rand::Rng;
use raylib::prelude::*;
use crate::blob::*;

const N_POP: usize = 80; 
const N_FOOD: usize = 240; 

pub struct Sim {
    food: Arc<Mutex<Vec<Food>>>,
    pub rl: RaylibHandle,
    rt: RaylibThread,
    border: Rectangle,
    gen_counter: u32,
    best_fitness: f32,
    is_paused: bool,
    algo: GenAlgo<Blob>
}

impl Sim {
    pub fn init() -> Self {
        let mut food = Vec::with_capacity(N_FOOD);
        for _ in 0..N_FOOD {
            food.push(Food::new());
        }

        let mut pops = Vec::with_capacity(N_POP);
        for _ in 0..N_POP {
            pops.push(Blob::new());
        }

        let (mut rl, rt) = raylib::init()
            .size(980, 640)
            .title("Genetic Algorithm Simulation")
            .build();
        rl.set_target_fps(240);

        let border = Rectangle {x: 60.0, y: 60.0, width: 860.0, height: 520.0};
        let ptr = Arc::new(Mutex::new(food));

        Sim { food: ptr, rl, rt, border, gen_counter: 1, is_paused: true, best_fitness: 0.0, algo: GenAlgo::<Blob>::new(pops) }
    }

    pub fn draw(&mut self) {
        let mut d = self.rl.begin_drawing(&self.rt);
        Sim::draw_bg(&mut d, self.border, self.gen_counter, self.best_fitness);

        for pop in &self.algo.pops {
            pop.draw(&mut d);
        } 

        let food = self.food.lock().unwrap();
        for i in 0..food.len() {
            food[i].draw(&mut d);
        }
    }

    // Probably terribly written
    pub fn update(&mut self) {
        self.handle_inputs();

        if self.is_paused {
            return;
        }

        let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(self.algo.pops.len());
        // What have I done
        let new_pops: Arc<Mutex<Vec<Blob>>> = Arc::new(Mutex::new(Vec::with_capacity(self.algo.pops.len()))); 

        for p in &self.algo.pops {
            let mut p_c = (*p).clone();
            let ptr = self.food.clone();
            let v = new_pops.clone();

            let handle = thread::spawn(move || {
                p_c.update(ptr);
                v.lock().unwrap().push(p_c);
            }); 

            handles.push(handle);
        } 

        for handle in handles {
             handle.join().expect("Failed to join thread");
        }

        let any_alive = new_pops.lock().unwrap().iter().any(|p| p.alive);

        self.algo.pops.clear(); 
        for p in new_pops.lock().unwrap().iter() {
            self.algo.pops.push(p.clone());
        }

        if !any_alive {
            self.gen_counter += 1;
            self.change_gen();
        }
    }

    fn handle_inputs(&mut self) {
        if self.rl.is_key_pressed(KeyboardKey::KEY_S) {
            self.save_state();
        }

        if self.rl.is_key_pressed(KeyboardKey::KEY_L) {
            self.load_state();
        }

        if self.rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            self.is_paused = !self.is_paused;
        }
    }

    fn draw_bg(d: &mut RaylibDrawHandle, r: Rectangle, counter: u32, score: f32) {
        d.clear_background(Color::BLACK);
        d.draw_rectangle_lines_ex(r, 1, Color::RAYWHITE);
        d.draw_fps(840, 30);
        d.draw_text(format!("Generation {}", counter).as_str(), 60, 20, 30, Color::RAYWHITE);
        d.draw_text(format!("Current Best: {}", score).as_str(), 360, 20, 30, Color::RAYWHITE);
        d.draw_text("Press S to save, L to load, Space to start/pause the simulation.", 60, 600, 20, Color::RAYWHITE)
    }

    fn change_gen(&mut self) {
        for p in &mut self.algo.pops {
            if p.fitness_fn() > self.best_fitness {
                self.best_fitness = p.fitness_fn();
            }
        }

        self.algo.evolve();

        self.food.lock().unwrap().clear();
        for _ in 0..N_FOOD {
            self.food.lock().unwrap().push(Food::new());
        }
    }

    fn save_state(&self) {
        let mut data_file = File::create("sim-state/data.txt").unwrap();

        _ = writeln!(data_file, "{}", self.gen_counter);
        _ = writeln!(data_file, "{}", self.best_fitness);

        for i in 0..self.algo.pops.len() {
            let mut file = File::create(format!("sim-state/{}.txt", i)).unwrap();

            for n in self.algo.pops[i].chromosome() {
                _ = writeln!(file, "{n}");
            }
        }
    }

    fn load_state(&mut self) {
        let mut blobs: Vec<Blob> = Vec::with_capacity(self.algo.pops.len());
        
        let file = File::open("sim-state/data.txt").unwrap();
        let reader = BufReader::new(file);
        let contents: Vec<f32> = reader.lines()
            .map(|line| {
                line.unwrap().trim().parse().unwrap()
            })
            .collect();
        self.gen_counter = contents[0] as u32;
        self.best_fitness = contents[1];

        for i in 0..self.algo.pops.len() {
            let file = File::open(format!("sim-state/{}.txt", i)).unwrap();
            let reader = BufReader::new(file);
            let mut strand: Vec<f32> = Vec::with_capacity(self.algo.pops[i].chromosome().len()); 

            for line in reader.lines() {
                strand.push(line.unwrap().trim().parse().unwrap());
            }

            blobs.push(Blob::from_strand(strand.as_slice()));
        }

        self.algo.pops = blobs;
    }
}

#[derive(Clone, Copy)]
pub struct Food {
    pub pos: Vector2,
    pub eaten: bool
}

impl Food {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        Food { pos: Vector2 { x: rng.gen_range(80.0..900.0), y: rng.gen_range(80.0..560.0) }, eaten: false }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        if !self.eaten {
            d.draw_circle(self.pos.x as i32, self.pos.y as i32, 2.0, Color::DARKGREEN);
        }
    }
}

