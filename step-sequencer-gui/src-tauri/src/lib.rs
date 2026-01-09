use std::{sync::Mutex, thread};

use step_sequencer::{
    beatmaker::{
        pattern::{ExampleDrumTracks, EXAMPLE_DRUMTRACKS_BITWIG, EXAMPLE_DRUMTRACKS_GARAGEBAND},
        BeatMakerEvent, BeatMakerSubscription,
    },
    drum_track::DrumTrack,
    id::SSId,
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

#[tauri::command]
fn set_tempo(state: State<Mutex<AppState>>, tempo: u16) -> String {
    print!("{}", tempo);
    state
        .lock()
        .unwrap()
        .ss_launcher
        .project()
        .project_settings()
        .write()
        .unwrap()
        .tempo = tempo;
    format!("set tempo to {}", tempo)
}

#[tauri::command]
fn get_track_list(state: State<Mutex<AppState>>) -> Vec<(String, DrumTrack)> {
    state
        .lock()
        .unwrap()
        .ss_launcher
        .project()
        .tracks()
        .read()
        .unwrap()
        .iter()
        .map(|(id, track)| (id.to_string(), track.clone()))
        .collect()
}

#[tauri::command]
fn add_empty_track(state: State<Mutex<AppState>>) -> SSId {
    state
        .lock()
        .unwrap()
        .ss_launcher
        .project()
        .add_empty_track()
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
            greet,
            play,
            pause,
            stop,
            get_tempo,
            set_tempo,
            get_track_list,
            add_empty_track
        ])
        .setup(|app| {
            let beatmaker_event_subscription = ss_launcher.subscribe_to_beatmaker();
            app.manage(Mutex::new(AppState { ss_launcher }));
            run_beatmaker_event_handler(app.handle().clone(), beatmaker_event_subscription);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
