pub mod xstuff;
use signal_hook::{
    consts::{SIGINT, SIGTERM, SIGUSR1},
    iterator::Signals,
};
use std::{
    error::Error,
    process::exit,
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use xstuff::WindowSystem;

const SEPARATOR: &str = " | ";

fn signalhandler(sig: i32, commands: Vec<Vec<&'static str>>) -> Vec<(usize, String)> {
    let mut commands_to_run: Vec<(usize, &str)> = Vec::new();
    for (i, j) in commands.iter().enumerate() {
        if j[3].parse().unwrap_or(0) == sig {
            commands_to_run.push((i, j[1]));
        }
    }
    let commands_run: Vec<(usize, String)> = commands_to_run
        .iter()
        .map(|(i, j)| {
            let val = Command::new("sh").arg("-c").arg(j).output();
            (
                i.clone(),
                match val {
                    Ok(x) => String::from_utf8(x.stdout).unwrap_or(String::new()),
                    Err(_) => String::new(),
                },
            )
        })
        .collect();
    commands_run
}

fn setupsignals(
    commands: Vec<Vec<&'static str>>,
    status_bar: Arc<Mutex<Vec<String>>>,
    tx: Sender<bool>,
) -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
    thread::spawn(move || {
        for _ in signals.forever() {
            exit(0);
        }
    });
    let signals: Vec<_> = commands
        .iter()
        .enumerate()
        .map(|(i, j)| (i, j[3].parse().unwrap_or(0)))
        .collect();
    let mut needed: Vec<i32> = Vec::new();
    for i in signals {
        if i.1 > 0 {
            needed.push(i.1 + SIGUSR1 - 1);
        }
    }
    let mut signals = Signals::new(&needed)?;
    thread::spawn(move || {
        for sig in signals.forever() {
            let thread_tx = tx.clone();
            let commands_run = signalhandler(sig - SIGUSR1 + 1, commands.clone());
            commands_run.iter().for_each(|(i, j)| {
                let mut val = j.clone();
                val.pop();
                let mut x = status_bar.lock().unwrap();
                x[*i] = val;
            });
            thread_tx.send(true).unwrap();
        }
    });
    Ok(())
}

fn getcmds(
    commands: Vec<Vec<&'static str>>,
    status_bar: Arc<Mutex<Vec<String>>>,
    cur_time: i32,
    tx: Sender<bool>,
) {
    let val: Vec<_> = commands
        .iter()
        .enumerate()
        .map(|(i, j)| {
            let interval = j[2].parse().unwrap_or(0);
            if cur_time == -1 {
                let val = Command::new("sh").arg("-c").arg(j[1]).output();
                let mut output = match val {
                    Ok(x) => String::from_utf8(x.stdout).unwrap_or(String::from("\n")),
                    Err(_) => String::from("\n".to_string()),
                };
                output.pop();
                let mut x = status_bar.lock().unwrap();
                x[i] = output;
                true
            } else {
                if interval == 0 {
                    false
                } else {
                    if cur_time % interval != 0 {
                        false
                    } else {
                        let val = Command::new("sh").arg("-c").arg(j[1]).output();
                        let mut output = match val {
                            Ok(x) => String::from_utf8(x.stdout).unwrap_or(String::from("\n")),
                            Err(_) => String::from("\n".to_string()),
                        };
                        output.pop();
                        let mut x = status_bar.lock().unwrap();
                        x[i] = output;
                        true
                    }
                }
            }
        })
        .collect();
    if !(val.iter().all(|i| *i == false)) {
        tx.send(true).unwrap();
    }
    thread::sleep(Duration::new(1, 0));
}

fn main() -> Result<(), Box<dyn Error>> {
    // All commands to run
    let commands: Vec<Vec<&'static str>> = vec![
        vec!["", "date '+%b %d (%a) %I:%M%p'", "30", "45"],
        vec!["", "cat /sys/class/power_supply/BAT0/capacity", "80", "46"],
    ];
    let count: usize = commands.len();

    // Opens a new x-window
    let window_system = WindowSystem::new();
    let status_bar = Arc::new(Mutex::new(vec![String::new(); count]));
    let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    setupsignals(commands.clone(), Arc::clone(&status_bar), tx.clone())?;
    let mut count = -1;
    loop {
        getcmds(commands.clone(), Arc::clone(&status_bar), count, tx.clone());
        if rx.try_recv().is_ok() {
            let x = status_bar.lock().unwrap();
            let name = x.join(SEPARATOR);
            window_system.draw(name);
            std::mem::drop(x);
        }
        count += 1;
    }
}
