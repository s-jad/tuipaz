use log::info;
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Padding, StatefulWidget, Widget, block::Title,
    }, layout::Alignment,
};

use crate::db::db_mac::NoteIdentifier;

use super::app::ComponentState;

#[derive(Debug, Clone, Copy)]
pub(crate) enum NoteListAction {
    LoadNote,
    LinkNote,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum NoteListMode {
    Sidebar,
    Fullscreen,
}


#[derive(Debug, Clone)]
pub(crate) struct SelectionStyle {
    pub(crate) highlight: Color,
    pub(crate) pointer: String,
    pub(crate) modifier: Modifier,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteListTheme {
    pub(crate) text: Color,
    pub(crate) title: Color,
    pub(crate) selection_style: SelectionStyle,
    pub(crate) borders: Color,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteList {
    pub(crate) selected: usize,
    pub(crate) note_identifiers: Vec<NoteIdentifier>,
    pub(crate) action: NoteListAction,
    pub(crate) state: ComponentState,
    pub(crate) mode: NoteListMode,
    pub(crate) theme: NoteListTheme
}

impl NoteList {
    pub(crate) fn new(
        note_identifiers: Vec<NoteIdentifier>,
        action: NoteListAction,
        state: ComponentState,
        theme: NoteListTheme,
    ) -> Self {
        let selected = 0;

        Self {
            selected,
            note_identifiers,
            action,
            state,
            mode: NoteListMode::Fullscreen,
            theme,
        }
    }

    pub(crate) fn prev(&mut self) {
        let nids_len = self.note_identifiers.len();
        // Guard against crashes if user has no notes
        if nids_len == 0 {
            return;
        }
        self.selected = self.selected.saturating_add(nids_len - 1) % nids_len;
    }

    pub(crate) fn next(&mut self) {
        let nids_len = self.note_identifiers.len();
        // Guard against crashes if user has no notes
        if nids_len == 0 {
            return;
        }
        self.selected = self.selected.saturating_add(1) % nids_len;
    }

    pub(crate) fn update(&mut self, new_nid: NoteIdentifier) {
        self.note_identifiers.push(new_nid);
    }

    pub(crate) fn replace(&mut self, replace_nid: NoteIdentifier) {
        self.note_identifiers
            .iter_mut()
            .find(|nid| nid.id == replace_nid.id)
            .expect("Note id should be present")
            .title = replace_nid.title;
    }

    pub(crate) fn remove(&mut self, note_id: i64) {
        let pos = self.note_identifiers
            .iter()
            .position(|nid| nid.id == note_id)
            .expect("Note should be in note_identifiers");

        self.note_identifiers.remove(pos);
    }

    pub(crate) fn set_state(&mut self, new_state: ComponentState) {
        self.state = new_state;
    }

    pub(crate) fn set_action(&mut self, new_action: NoteListAction) {
        self.action = new_action;
    }

    pub(crate) fn set_mode(&mut self, new_mode: NoteListMode) {
        self.mode = new_mode;
    }
}

impl Widget for NoteList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (title_text, info_text, borders, padding) = match (self.mode, self.action) {
            (NoteListMode::Fullscreen, NoteListAction::LoadNote) => (
                " Load Note ",
                " <Esc> prev screen | <Enter> Load Note | <up/j> Next | <down/k> Prev ",
                Borders::ALL,
                Padding::new(1, 1, 1, 1),
            ),
            (NoteListMode::Fullscreen, NoteListAction::LinkNote) => (
                " Link Note ",
                " <Enter> Link Note | <up/j> Next | <down/k> Prev ",
                Borders::ALL,
                Padding::new(1, 1, 1, 1),
            ),
            (NoteListMode::Sidebar, _) => (
                " File Explorer ",
                " <Alt-f> hide files ",
                Borders::TOP | Borders::RIGHT | Borders::BOTTOM,
                Padding::new(1, 1, 0, 0),
            )
        };

        let (
            border_style,
            title_style,
            list_info_style,
            list_item_style,
            highlight_clr
        ) = match (self.state, self.mode) {
            (ComponentState::Active, NoteListMode::Fullscreen) => (
                Style::default().bold().fg(self.theme.borders),
                Style::default().bold().fg(self.theme.title),
                Style::default().bold(),
                Style::default(),
                Color::Red,
            ),
            (ComponentState::Inactive, NoteListMode::Fullscreen) => (
                Style::default().bold().fg(self.theme.borders).dim(),
                Style::default().bold().fg(self.theme.title).dim(),
                Style::default().bold().dim(),
                Style::default().dim(),
                Color::default(),
            ),
            (ComponentState::Unavailable, NoteListMode::Fullscreen) => (
                Style::default().dim(),
                Style::default().dim(),
                Style::default().dim(),
                Style::default().dim(),
                Color::default(),
            ),
            (ComponentState::Error, NoteListMode::Fullscreen) => (
                Style::default().bold().fg(self.theme.borders),
                Style::default().bold().fg(Color::Red),
                Style::default().bold().fg(Color::Red),
                Style::default(),
                Color::Red,
            ),
            (ComponentState::Active, NoteListMode::Sidebar) => (
                Style::default().bold().fg(self.theme.borders),
                Style::default().bold().fg(self.theme.title),
                Style::default().bold(),
                Style::default(),
                Color::Red,
            ),
            (ComponentState::Inactive, NoteListMode::Sidebar) => (
                Style::default().bold().fg(self.theme.borders).dim(),
                Style::default().bold().fg(self.theme.title).dim(),
                Style::default().bold().dim(),
                Style::default().dim(),
                Color::default(),
            ),
            _ => (
                Style::default(),
                Style::default(),
                Style::default(),
                Style::default(),
                Color::Red,
            )
        };

        let info_line = Line::styled(info_text, list_info_style).alignment(Alignment::Center);
        let title = Span::styled(title_text, title_style);
        
        let load_note_block = Block::default()
            .title(Title::from(title).alignment(Alignment::Center))
            .title_bottom(info_line)
            .padding(padding)
            .borders(borders)
            .border_type(BorderType::Rounded)
            .border_style(border_style);

        let mut state = ListState::default().with_selected(Some(self.selected));

        let list = List::from_iter(
            self.note_identifiers
                .into_iter()
                .map(|nid| ListItem::new(Line::from(nid.title)).style(list_item_style)),
        )
        .block(load_note_block)
        .highlight_style(
            Style::default()
                .add_modifier(self.theme.selection_style.modifier)
                .fg(self.theme.selection_style.highlight)
        )
        .highlight_symbol(&self.theme.selection_style.pointer)
        .repeat_highlight_symbol(true);

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
