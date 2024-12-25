use std::{
    rc::Rc,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Gauge, List, ListItem, Padding, Paragraph},
    Frame,
};
use step_sequencer::{error::SSError, project::Project, SSResult};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

pub(crate) struct Tui<'a> {
    input: Input,
    input_mode: InputMode,
    error: Option<SSError>,
    logs: Vec<String>,
    project: &'a Project,
}

impl<'a> Tui<'a> {
    pub fn new(project: &'a Project) -> Self {
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
}

impl<'a> Tui<'a> {
    pub fn run_tui<F>(&mut self, log_receiver: Receiver<String>, command_handler: F) -> SSResult<()>
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
        thread::spawn(move || loop {
            let event = event::read().unwrap();
            event_sender.send(TuiEvent::TerminalEvent(event));
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
                                    KeyCode::Enter => {
                                        self.switch_to_command_palette_mode();
                                    }
                                    _ => {}
                                },
                                InputMode::CommandPalette => match key.code {
                                    KeyCode::Enter => {
                                        let command = self.input.value();
                                        match command {
                                            "q" => break,
                                            "" => self.switch_to_normal_mode(),
                                            _ => match command_handler(self.input.value()) {
                                                Ok(()) => {
                                                    self.input.reset();
                                                }
                                                Err(e) => {
                                                    self.set_command_error(e);
                                                }
                                            },
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
                }
            }
        }
        ratatui::restore();
        Ok(())
    }

    fn process_command(&self, s: String) {}

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
        let track_count = { self.project.tracks().read().unwrap().len() };
        let operation_layout = Layout::vertical(vec![Constraint::Fill(1); track_count]);
        let track_areas = operation_layout.split(operation_area);

        for (idx, gauge) in self.track_widgets().iter().enumerate() {
            frame.render_widget(gauge, track_areas[idx]);
        }
        frame.render_widget(self.log_widget(), logging_area);
        frame.render_widget(self.command_area_widget(), command_area);
    }

    fn track_widgets(&self) -> Vec<Gauge> {
        let binding = self.project.tracks();
        let tracks = binding.read().unwrap();
        let current_beats = {
            let binding = self.project.project_settings();
            let binding = binding.read().unwrap();
            let x = *binding.current_beats.read().unwrap();
            x
        };
        tracks
            .values()
            .map(|track| {
                let total = track.total_beats();
                let current = (current_beats as usize) % total;
                Gauge::default()
                    .gauge_style(Color::Rgb(48, 48, 84))
                    .percent((current * 100 / total) as u16)
            })
            .collect()
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
