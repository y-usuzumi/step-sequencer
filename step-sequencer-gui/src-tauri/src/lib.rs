use std::{sync::Mutex, thread};

use crossbeam::channel::Receiver;
use step_sequencer::{
    beatmaker::{
        pattern::{ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND},
        BeatSignal,
    },
    launcher::SSLauncher,
    SSResult,
};
use tauri::{AppHandle, Emitter, Manager, State};

pub(crate) struct AppState {
    ss_launcher: SSLauncher,
}

fn create_step_sequencer() -> SSResult<SSLauncher> {
    let ss_launcher = SSLauncher::new();
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
fn play(state: State<Mutex<AppState>>) -> String {
    state.lock().unwrap().ss_launcher.timeline().start();
    format!("Playing")
}

#[tauri::command]
fn pause(state: State<Mutex<AppState>>) -> String {
    state.lock().unwrap().ss_launcher.timeline().pause();
    format!("Paused")
}

#[tauri::command]
fn stop(state: State<Mutex<AppState>>) -> String {
    state.lock().unwrap().ss_launcher.timeline().stop();
    format!("Stopped")
}

#[tauri::command]
fn get_tempo(state: State<Mutex<AppState>>) -> u16 {
    state
        .lock()
        .unwrap()
        .ss_launcher
        .project()
        .project_settings()
        .read()
        .unwrap()
        .tempo
}

fn run_beat_signal_handler(app_handle: AppHandle, beat_signal_receiver: Receiver<BeatSignal>) {
    thread::spawn(move || {
        for beat_signal in beat_signal_receiver.iter() {
            app_handle.emit("beat-signal", beat_signal).unwrap();
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut ss_launcher = create_step_sequencer().unwrap();
    ss_launcher.start().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, play, pause, stop, get_tempo
        ])
        .setup(|app| {
            let beat_signal_receiver = ss_launcher.subscribe_beatmaker_signals();
            app.manage(Mutex::new(AppState {
                ss_launcher: ss_launcher,
            }));
            run_beat_signal_handler(app.handle().clone(), beat_signal_receiver);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
