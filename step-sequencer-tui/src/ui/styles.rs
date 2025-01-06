use std::sync::OnceLock;

use ratatui::style::{
    palette::{self, tailwind},
    Color,
};

pub(crate) struct TrackerViewStyles {
    pub default_row_bg: Color,
    pub default_cell_bg: Color,
    pub current_playing_row_bg: Color,
    pub current_playing_cell_bg: Color,
}

pub fn get_tracker_view_styles() -> &'static TrackerViewStyles {
    static tracker_view_styles: OnceLock<TrackerViewStyles> = OnceLock::new();
    tracker_view_styles.get_or_init(|| TrackerViewStyles {
        default_cell_bg: tailwind::BLUE.c900,
        default_row_bg: tailwind::BLUE.c900,
        current_playing_row_bg: tailwind::BLUE.c400,
        current_playing_cell_bg: tailwind::BLUE.c400,
    })
}
