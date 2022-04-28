use anyhow::Result;
use crossbeam::thread;
use std::time;

pub trait Runnable: Sync + Send {
    fn run(&self) -> Result<()>;
}

pub fn run_in_parallel(things_to_run: Vec<Box<dyn Runnable>>) {
    thread::scope(|s| {
        for thing in things_to_run {
            s.spawn(move |_| {
                thing.run().unwrap();
            });
        }
    })
    .unwrap();
}

// Suspend the execution of the thread by a particular amount of milliseconds
pub fn sleep_millis(millis: u64) {
    let wait_duration = time::Duration::from_millis(millis);
    std::thread::sleep(wait_duration);
}

// Quit the program when the user inputs Ctrl-C
pub fn set_ctrlc_handler() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}
