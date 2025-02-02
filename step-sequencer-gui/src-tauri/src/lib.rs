use std::{sync::Mutex, thread};

use step_sequencer::{
    beatmaker::{
        pattern::{ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND},
        BeatMakerEvent, BeatMakerSubscription,
    },
    launcher::{SSLauncher, SSLauncherImpl2},
    SSResult,
};
use tauri::{AppHandle, Emitter, Manager, State};

pub(crate) struct AppState<L> {
    ss_launcher: L,
}

fn create_step_sequencer() -> SSResult<impl SSLauncher> {
    let ss_launcher = SSLauncherImpl2::new();
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
fn play(state: State<Mutex<AppState<Box<dyn SSLauncher>>>>) -> String {
    state.lock().unwrap().ss_launcher.start();
    format!("Playing")
}

#[tauri::command]
fn pause(state: State<Mutex<AppState<Box<dyn SSLauncher>>>>) -> String {
    state.lock().unwrap().ss_launcher.pause();
    format!("Paused")
}

#[tauri::command]
fn stop(state: State<Mutex<AppState<Box<dyn SSLauncher>>>>) -> String {
    state.lock().unwrap().ss_launcher.stop();
    format!("Stopped")
}

#[tauri::command]
fn get_tempo(state: State<Mutex<AppState<Box<dyn SSLauncher>>>>) -> u16 {
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

fn run_beatmaker_event_handler(
    app_handle: AppHandle,
    beatmaker_subscription: BeatMakerSubscription,
) {
    thread::spawn(move || {
        for event in beatmaker_subscription.receiver.iter() {
            match event {
                BeatMakerEvent::Beat(_) | BeatMakerEvent::Pause | BeatMakerEvent::Stop => {
                    app_handle.emit("beat-signal", event).unwrap();
                }
                _ => {}
            }
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
            let beatmaker_event_subscription = ss_launcher.subscribe_to_beats();
            app.manage(Mutex::new(AppState {
                ss_launcher: Box::new(ss_launcher),
            }));
            run_beatmaker_event_handler(app.handle().clone(), beatmaker_event_subscription);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
