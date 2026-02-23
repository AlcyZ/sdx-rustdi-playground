use crate::scheduler::{Res, Scheduler};

mod scheduler;

fn main() {
    let mut scheduler = Scheduler::new();

    scheduler.add_system(sample_system);
    scheduler.add_resource(512i32);
    scheduler.add_resource(1024i64);

    scheduler.run();
}

fn sample_system(arg1: Res<i32>, arg2: Res<i64>) {
    println!("passed to sample system: {} and {}", arg1.value, arg2.value)
}
