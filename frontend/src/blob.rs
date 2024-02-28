use std::sync::{Arc, Mutex};
use genalgo::*;
use nnet::*;
use rand::Rng;
use raylib::prelude::*;
use crate::sim::Food;

pub struct Blob {
    pub pos: Vector2,
    net: Network,
    pub alive: bool,
    energy: f32,
    n_eaten: u32,
    sensors: [Ray; 8]
} 

impl Pop for Blob {
    fn new() -> Self {
        let mut rng = rand::thread_rng();

        let pos = Vector2 {x: rng.gen_range(80..900) as f32, y: rng.gen_range(80..560) as f32};
        let net = Network::new(&[8, 3, 3, 3, 2]);

        let sensors = [
            Ray {position: Vector3 {x: pos.x, y: pos.y, z: 0.0}, direction: Vector3 {x: 0.0, y: -1.0, z: 0.0}},
            Ray {position: Vector3 {x: pos.x, y: pos.y, z: 0.0}, direction: Vector3 {x: 1.0, y: -1.0, z: 0.0}},
            Ray {position: Vector3 {x: pos.x, y: pos.y, z: 0.0}, direction: Vector3 {x: 1.0, y: 0.0, z: 0.0}},
            Ray {position: Vector3 {x: pos.x, y: pos.y, z: 0.0}, direction: Vector3 {x: 1.0, y: 1.0, z: 0.0}},
            Ray {position: Vector3 {x: pos.x, y: pos.y, z: 0.0}, direction: Vector3 {x: 0.0, y: 1.0, z: 0.0}},
            Ray {position: Vector3 {x: pos.x, y: pos.y, z: 0.0}, direction: Vector3 {x: -1.0, y: 1.0, z: 0.0}},
            Ray {position: Vector3 {x: pos.x, y: pos.y, z: 0.0}, direction: Vector3 {x: -1.0, y: 0.0, z: 0.0}},
            Ray {position: Vector3 {x: pos.x, y: pos.y, z: 0.0}, direction: Vector3 {x: -1.0, y: -1.0, z: 0.0}},
        ];

        Blob { pos, net, alive: true, energy: 60.0, n_eaten: 0, sensors }
    }

    fn chromosome(&self) -> Vec<f32> {
        self.net.extract() 
    }

    fn fitness_fn(&self) -> f32 {
        f32::max(self.n_eaten as f32, 0.1)
    }

    fn from_strand(strand: &[f32]) -> Self {
        let mut b = Blob::new(); 
        b.net.rebuild(strand);
        b
    }
}

impl Clone for Blob {
    fn clone(&self) -> Self {
        let mut nnet = Network::new(&[8, 3, 3, 3, 2]);
        let seq = self.net.extract();
        nnet.rebuild(&seq);

        Blob { pos: self.pos, net: nnet, alive: self.alive, energy: self.energy, n_eaten: self.n_eaten, sensors: self.sensors }
    }
}

impl Blob {
    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        if self.alive {
            d.draw_circle(self.pos.x as i32, self.pos.y as i32, 10.0, Color::RAYWHITE);
        }
    }

    pub fn update(&mut self, ptr: Arc<Mutex<Vec<Food>>>) {
        if !self.alive {
            return
        }

        let mut data: Vec<f32> = vec![0.0; self.sensors.len()];

        let mut counter = 0;
        let mut min_distance = f32::MAX;
        for ray in &self.sensors {
            let food = ptr.lock().unwrap();
            for i in 0..food.len() {
                if food[i].eaten {
                    continue;
                }

                if check_collision_ray_sphere(*ray, Vector3{x: food[i].pos.x, y: food[i].pos.y, z: 0.0}, 5.0) {
                    let distance = f32::sqrt(f32::abs((self.pos.x - food[i].pos.x).powi(2) + (self.pos.y - food[i].pos.y).powi(2)));
                    if distance <= min_distance {
                        _ = std::mem::replace(&mut data[counter], distance);
                        min_distance = distance;
                    }
                } 
            }
            counter += 1;
        }

        let v_res = &self.net.propagate(&data);
        let v_res = &[f32::min(v_res[0], 5.0), f32::min(v_res[1], 5.0)];
        let v_res = &[f32::max(v_res[0], -5.0), f32::max(v_res[1], -5.0)];
        
        if self.pos.x + v_res[0] < 860.0 && self.pos.x + v_res[0] > 60.0 {
            self.pos.x += v_res[0]; 
        } 

        if self.pos.y + v_res[1] < 520.0 && self.pos.y + v_res[1] > 60.0 {
            self.pos.y += v_res[1]; 
        }

        let mag = f32::sqrt(v_res[0].powi(2) + v_res[1].powi(2));

        // penalty for speed
        self.energy -= 1.0/24.0 * f32::max(mag, 1.0); 
        
        let temp = ptr.lock().unwrap().clone();
        counter = 0;
        for f in temp {
            if (!f.eaten) && check_collision_circles(f.pos, 5.0, self.pos, 10.0) {
                ptr.lock().unwrap()[counter].eaten = true;
                self.n_eaten += 1;
                self.energy = 60.0;
            }
            counter += 1;
        }
       
        if self.energy <= 0.0 {
            self.alive = false;
        }
    }
}
