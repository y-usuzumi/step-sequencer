use std::{
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crossterm::event::{self, Event, KeyCode};
use log::info;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};
use step_sequencer::{
    beatmaker::BeatSignal,
    drum_track::{DrumTrack, DrumTrackBeat},
    error::SSError,
    project::Project,
    SSResult,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::widgets::BeatPad;

pub(crate) struct Tui {
    input: Input,
    input_mode: InputMode,
    error: Option<SSError>,
    logs: Vec<String>,
    project: Rc<Project>,
}

impl Tui {
    pub fn new(project: Rc<Project>) -> Self {
        Self {
            input: Input::default(),
            input_mode: InputMode::Normal,
            error: None,
            logs: Vec::new(),
            project,
        }
    }
}

enum InputMode {
    Normal,
    CommandPalette,
}

pub(crate) struct TuiLogger {
    sender: Sender<String>,
}

impl TuiLogger {
    pub fn new(sender: Sender<String>) -> Self {
        Self { sender }
    }
}

impl log::Log for TuiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let _ = self
                .sender
                .send(format!("[{}] {}", record.level(), record.args()));
        }
    }

    fn flush(&self) {
        // No-op
    }
}

enum TuiEvent {
    LogEvent(String),
    TerminalEvent(Event),
    Redraw,
}

impl Tui {
    pub fn run_tui<F>(
        &mut self,
        beat_signal_receiver: Receiver<BeatSignal>,
        log_receiver: Receiver<String>,
        command_handler: F,
    ) -> SSResult<()>
    where
        F: Fn(&str) -> SSResult<()>,
    {
        let mut terminal = ratatui::init();
        let (event_sender, event_receiver) = mpsc::channel();
        {
            let event_sender = event_sender.clone();
            thread::spawn(move || {
                for log in log_receiver {
                    event_sender.send(TuiEvent::LogEvent(log));
                }
            });
        }
        {
            let event_sender = event_sender.clone();
            thread::spawn(move || loop {
                let event = event::read().unwrap();
                event_sender.send(TuiEvent::TerminalEvent(event));
            });
        }
        thread::spawn(move || loop {
            if let Ok(signal) = beat_signal_receiver.recv() {
                match signal {
                    _ => {
                        // Be it Beat, Pause or Stop, redraw anyways
                        event_sender.send(TuiEvent::Redraw);
                    }
                }
            }
        });

        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .expect("failed to draw frame");

            if let Ok(event) = event_receiver.recv() {
                match event {
                    TuiEvent::LogEvent(log) => {
                        self.append_log(log);
                    }
                    TuiEvent::TerminalEvent(event) => {
                        if let Event::Key(key) = event {
                            match self.input_mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('q') => {
                                        break;
                                    }
                                    KeyCode::Char(':') => {
                                        self.switch_to_command_palette_mode();
                                    }
                                    KeyCode::Char(' ') => {
                                        self.execute_command("play", &command_handler);
                                    }
                                    KeyCode::Esc => {
                                        self.execute_command("stop", &command_handler);
                                    }
                                    _ => {}
                                },
                                InputMode::CommandPalette => match key.code {
                                    KeyCode::Enter => {
                                        let command = self.input.value().to_string();
                                        match command.as_str() {
                                            "q" => break,
                                            "" => self.switch_to_normal_mode(),
                                            _ => self.execute_command(&command, &command_handler),
                                        }
                                    }
                                    KeyCode::Esc => {
                                        self.switch_to_normal_mode();
                                    }
                                    _ => {
                                        self.clear_command_error();
                                        self.input.handle_event(&Event::Key(key));
                                    }
                                },
                            }
                        }
                    }
                    TuiEvent::Redraw => {}
                }
            }
        }
        ratatui::restore();
        Ok(())
    }

    fn execute_command<F>(&mut self, command: &str, command_handler: &F)
    where
        F: Fn(&str) -> SSResult<()>,
    {
        match command_handler(command) {
            Ok(()) => {
                self.input.reset();
            }
            Err(e) => {
                self.set_command_error(e);
            }
        }
    }

    fn append_log(&mut self, log: String) {
        self.logs.push(log);
    }

    fn switch_to_command_palette_mode(&mut self) {
        self.input_mode = InputMode::CommandPalette;
    }

    fn switch_to_normal_mode(&mut self) {
        self.clear_command_error();
        self.input.reset();
        self.input_mode = InputMode::Normal;
    }

    fn set_command_error(&mut self, err: SSError) {
        self.error = Some(err);
    }

    fn clear_command_error(&mut self) {
        self.error = None;
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Length(3),
        ]);
        let [operation_area, logging_area, command_area] = vertical.areas(frame.area());
        if self.logs.len() > (logging_area.height - 2) as usize {
            self.logs
                .drain(0..(self.logs.len() - (logging_area.height - 2) as usize));
        }
        let operation_area_block = Block::new()
            .borders(Borders::ALL)
            .title("Tracks")
            .padding(Padding::uniform(1));
        frame.render_widget(&operation_area_block, operation_area);
        let operation_area = operation_area_block.inner(operation_area);
        let binding = self.project.tracks();
        let tracks = binding.read().unwrap();
        let operation_layout = Layout::vertical(vec![Constraint::Fill(1); tracks.len()]);
        let track_areas = operation_layout.split(operation_area);
        let current_beat = {
            let binding = self.project.project_settings();
            let binding = binding.read().unwrap();
            let x = *binding.current_beats.read().unwrap();
            x
        };

        for (track, area) in tracks.values().zip(track_areas.into_iter()) {
            self.render_track(frame, track, current_beat, *area);
        }
        frame.render_widget(self.log_widget(), logging_area);
        frame.render_widget(self.command_area_widget(), command_area);
    }

    fn render_track(
        &self,
        frame: &mut Frame,
        track: &DrumTrack,
        current_beat: u64,
        operation_area: Rect,
    ) {
        let border = Block::new().borders(Borders::ALL).title(track.name());
        frame.render_widget(&border, operation_area);
        let operation_area = border.inner(operation_area);
        let total_beats = track.total_beats();
        let horizontal = Layout::horizontal(vec![Constraint::Fill(1); total_beats]);
        let areas = horizontal.split(operation_area);
        let active_idx = (current_beat as usize) % total_beats;
        for idx in 0..total_beats {
            let block = Block::new().padding(Padding::uniform(1));
            frame.render_widget(&block, areas[idx]);
            let area = block.inner(areas[idx]);
            let is_active_beat = idx == active_idx;
            let color = match track.get(idx) {
                DrumTrackBeat::Unset => {
                    if is_active_beat {
                        Color::DarkGray
                    } else {
                        Color::Black
                    }
                }
                DrumTrackBeat::DefaultBeat => {
                    if is_active_beat {
                        Color::LightBlue
                    } else {
                        Color::LightMagenta
                    }
                }
                DrumTrackBeat::OverrideBeat(_) => {
                    if is_active_beat {
                        Color::LightBlue
                    } else {
                        Color::LightRed
                    }
                }
            };
            let widget = BeatPad::color(color).set_active(is_active_beat);

            frame.render_widget(widget, area);
        }
    }

    fn log_widget(&self) -> List {
        let messages: Vec<ListItem> = self
            .logs
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Line::from(Span::raw(format!("{}: {}", i, m)))];
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Log"));
        return messages;
    }

    fn get_input_mode_style(&self) -> Style {
        match self.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::CommandPalette => {
                if self.error.is_some() {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Yellow)
                }
            }
        }
    }

    fn command_area_widget(&self) -> Paragraph {
        let title = match self.error {
            Some(ref err) => format!("Command ({})", err),
            None => "Command".to_string(),
        };
        let paragraph = Paragraph::new(self.input.value())
            .style(self.get_input_mode_style())
            .block(
                Block::bordered()
                    .title(title)
                    .padding(Padding::symmetric(1, 0))
                    .style(self.get_input_mode_style()),
            );
        return paragraph;
    }
}
