use std::{
    cell::RefCell,
    sync::{Arc, LazyLock, OnceLock, RwLock},
    thread,
};

use step_sequencer::{
    beatmaker::pattern::{
        ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND,
    },
    launcher::SSLauncher,
    SSResult,
};

static mut SS_LAUNCHER: LazyLock<RefCell<SSLauncher>> =
    std::sync::LazyLock::new(|| RefCell::new(create_step_sequencer().unwrap()));

fn create_step_sequencer() -> SSResult<SSLauncher> {
    let mut ss_launcher = SSLauncher::new();
    let project = ss_launcher.project();
    let example_drumtracks = if cfg!(target_os = "linux") {
        &EXAMPLE_DRUMTRACKS_BITWIG
    } else {
        &EXAMPLE_DRUMTRACKS_GARAGEBAND
    };
    for track in example_drumtracks.all_tracks() {
        project.add_track(track);
    }
    Ok(ss_launcher)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn play() -> String {
    unsafe {
        SS_LAUNCHER.borrow().timeline().start();
    }
    format!("Playing")
}

#[tauri::command]
fn pause() -> String {
    unsafe {
        SS_LAUNCHER.borrow().timeline().pause();
    }
    format!("Paused")
}

#[tauri::command]
fn stop() -> String {
    unsafe {
        SS_LAUNCHER.borrow().timeline().stop();
    }
    format!("Stopped")
}

#[tauri::command]
fn get_tempo() -> u16 {
    unsafe {
        SS_LAUNCHER
            .borrow()
            .project()
            .project_settings()
            .read()
            .unwrap()
            .tempo
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    unsafe {
        SS_LAUNCHER.borrow_mut().start();
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, play, pause, stop, get_tempo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
