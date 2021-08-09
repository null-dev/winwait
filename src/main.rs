#![windows_subsystem = "windows"]

mod process;

use std::path::Path;
use crate::process::for_each_process;
use serde::Deserialize;
use std::env::args;
use std::thread::sleep;
use std::time::Duration;
use std::ffi::{OsString, OsStr};
use std::path::Component::Normal;
use std::process::Command;
use std::fs::OpenOptions;

#[derive(Deserialize, Debug)]
struct Config {
    cmd_on_open: Vec<String>,
    cmd_on_close: Vec<String>,
    targets: Vec<String>
}

fn main() {
    let mut args = args();
    if args.next().is_none() {
        println!("Must pass path to config file!");
        return
    }
    let config: Config;
    if let Some(str) = args.next() {
        let path = Path::new(&str);
        if let Ok(file) = OpenOptions::new().read(true).open(path) {
            match serde_json::from_reader(file) {
                Ok(res) => config = res,
                Err(e) => {
                    println!("Could not load config file: {:?}", e);
                    return
                }
            }
        } else {
            println!("Cannot open config file!");
            return
        }
    } else {
        println!("Must pass path to config file!");
        return
    }

    let targets_as_os_str: Vec<OsString> = config.targets
        .iter()
        .map(|f| OsString::from(f))
        .collect();

    println!("BOOTED!");

    let mut running = false;
    loop {
        let mut new_running = false;
        for_each_process(|_id, name| {
            if let Some(Normal(exe)) = name.components().last() {
                if targets_as_os_str.iter().any(|f| f == exe) {
                    new_running = true;
                }
            }
        });
        if running != new_running {
            println!("New running status: {}", new_running);
            let cmd_str: &Vec<String>;
            if new_running {
                cmd_str = &config.cmd_on_open;
            } else {
                cmd_str = &config.cmd_on_close;
            }

            if !cmd_str.is_empty() {
                let mut cmd = Command::new(cmd_str.first().unwrap());
                for arg in cmd_str.iter().skip(1) {
                    cmd.arg(arg);
                }
                cmd.spawn().and_then(|mut f| f.wait());
            }

            println!("Command executed!");
            running = new_running;
        }

        sleep(Duration::new(3, 0));
    }
}
