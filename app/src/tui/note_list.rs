use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Padding, StatefulWidget, Widget,
    },
};

use crate::db::db_mac::NoteIdentifier;

#[derive(Debug, Clone)]
pub(crate) enum NoteListState {
    Active,
    Inactive,
}

#[derive(Debug, Clone)]
pub(crate) enum NoteListAction {
    LoadNote,
    LinkNote,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteList {
    pub(crate) selected: usize,
    pub(crate) note_identifiers: Vec<NoteIdentifier>,
    pub(crate) action: NoteListAction,
    pub(crate) state: NoteListState,
}

impl NoteList {
    pub(crate) fn new(
        note_identifiers: Vec<NoteIdentifier>,
        action: NoteListAction,
        state: NoteListState,
    ) -> Self {
        let selected = 0;

        Self {
            selected,
            note_identifiers,
            action,
            state,
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

    pub(crate) fn set_state(&mut self, new_state: NoteListState) {
        self.state = new_state;
    }

    pub(crate) fn set_action(&mut self, new_action: NoteListAction) {
        self.action = new_action;
    }
}

impl Widget for NoteList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (title_text, list_info_text) = match self.action {
            NoteListAction::LoadNote => (
                " Load Note ",
                " <Enter> Load Note | <ArrowUp/j> Next | <ArrowDown/k> Prev ",
            ),
            NoteListAction::LinkNote => (
                " Link Note ",
                " <Enter> Link Note | <ArrowUp/j> Next | <ArrowDown/k> Prev ",
            ),
        };

        let (border_style, title_style, list_info_style) = match self.state {
            NoteListState::Active => (
                Style::default().bold(),
                Style::default().bold().fg(Color::Yellow),
                Style::default().bold(),
            ),
            NoteListState::Inactive => (
                Style::default().dim(),
                Style::default().dim(),
                Style::default().dim(),
            ),
        };

        let list_info = Line::styled(list_info_text, list_info_style);

        let title = Span::styled(title_text, title_style);

        let load_note_block = Block::default()
            .title(title)
            .title_bottom(list_info)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .padding(Padding::new(1, 1, 1, 1))
            .style(Style::default());

        let mut state = ListState::default().with_selected(Some(self.selected));

        let list = List::from_iter(
            self.note_identifiers
                .into_iter()
                .map(|nid| ListItem::new(Line::from(nid.title))),
        )
        .block(load_note_block)
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true);

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
