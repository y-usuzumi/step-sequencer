use log::info;
use ratatui::{
    layout::Constraint,
    style::Stylize,
    widgets::{Cell, Row, StatefulWidget, Table, TableState},
};
use step_sequencer::{drum_track::Beat, project::TrackMap};

use super::styles::get_tracker_view_styles;

pub struct TrackerViewState {
    table_state: TableState,
    playing_row: usize,
}

impl Default for TrackerViewState {
    fn default() -> Self {
        Self {
            table_state: Default::default(),
            playing_row: 0,
        }
    }
}

pub struct TrackerView<'a> {
    tracks: &'a TrackMap,
    current_beat: u64,
}

impl<'a> TrackerView<'a> {
    pub fn new(tracks: &'a TrackMap, current_beat: u64) -> Self {
        Self {
            tracks,
            current_beat,
        }
    }
}

fn display_beat(beat: &Option<Beat>) -> String {
    match *beat {
        None => "".to_string(),
        Some(beat) => beat.note.to_string(),
    }
}

impl<'a> StatefulWidget for TrackerView<'a> {
    type State = TrackerViewState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let styles = get_tracker_view_styles();
        let headers = self.tracks.values().map(|t| t.name());
        let mut rows = vec![];
        let track_count = self.tracks.len();
        for (track_idx, track) in self.tracks.values().enumerate() {
            let active_beat_idx = (self.current_beat as usize) % track.len();
            for (beat_idx, beat) in track.iter_as_beats().enumerate() {
                if rows.len() < track.len() {
                    rows.resize(track.len() + 1, vec![Cell::new(""); track_count]);
                }
                let row_bg_style = if active_beat_idx == beat_idx {
                    styles.current_playing_cell_bg
                } else {
                    styles.default_cell_bg
                };
                rows[beat_idx][track_idx] = Cell::new(display_beat(&beat)).bg(row_bg_style);
            }
        }
        let rows: Vec<Row> = rows.into_iter().map(|r| Row::new(r)).collect();
        let table = Table::new(rows, vec![Constraint::Fill(1); self.tracks.len()])
            .header(Row::new(headers));

        table.render(area, buf, &mut state.table_state);
    }
}
