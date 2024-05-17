use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Padding, StatefulWidget, Widget,
    },
};

use crate::db::db_mac::NoteIdentifier;

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
}

impl NoteList {
    pub(crate) fn new(note_identifiers: Vec<NoteIdentifier>, action: NoteListAction) -> Self {
        let selected = 0;

        Self {
            selected,
            note_identifiers,
            action,
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
                " <Enter> Load Note | <Tab/ArrowUp> Next | <Shift-Tab/ArrowDown> Prev ",
            ),
            NoteListAction::LinkNote => (
                " Link Note ",
                " <Enter> Link Note | <Tab/ArrowUp> Next | <Shift-Tab/ArrowDown> Prev ",
            ),
        };

        let list_info = Line::styled(list_info_text, Style::default().fg(Color::Red));

        let title = Span::styled(title_text, Style::default().bold().fg(Color::Yellow));

        let load_note_block = Block::default()
            .title(title)
            .title_bottom(list_info)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
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
