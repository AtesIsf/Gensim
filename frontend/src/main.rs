pub mod sim;
pub mod blob;

use sim::*;

fn main() {
    let mut sim = Sim::init();

    while !sim.rl.window_should_close() {
        sim.update();
        sim.draw();
    }
}
