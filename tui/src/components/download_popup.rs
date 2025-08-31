use client_core::Attachment;
use color_eyre::Result;
use crossterm::event::KeyCode;
use itertools::Itertools;
use layout::Flex;
use ratatui::{prelude::*, widgets::*};
use style::palette::tailwind::SLATE;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use super::Component;
use crate::{action::Action, app::Mode, config::Config};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

#[derive(Default)]
pub struct Popup {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
    enabled: bool,
    list: AttachmentList,
    visible: bool,
}

#[derive(Default)]
pub struct AttachmentList {
    list_items: Vec<AttachmentListItem>,
    state: ListState,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AttachmentListItem {
    attachment: Attachment,
    selected: bool,
}

impl Popup {
    pub fn new() -> Self {
        Popup {
            mode: Mode::CurrentAssignmentScreen,
            enabled: true,
            visible: false,
            ..Default::default()
        }
    }
}

impl Component for Popup {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {}
            Action::Mode(mode) => {
                self.enabled = mode == self.mode;
            }
            Action::ToggleDownloadPopup => {
                self.visible = !self.visible;
            }
            Action::Attachments(attachments) => {
                self.list =
                    AttachmentList::from_iter(attachments.into_iter().map(AttachmentListItem::new));
            }
            Action::FinishDownload => {
                self.command_tx
                    .clone()
                    .unwrap()
                    .send(Action::ToggleDownloadPopup)?;
            }
            _ => {}
        }
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        info!("Got key: {key:?}");

        if !self.enabled || !self.visible {
            return Ok(None);
        }
        match key.code {
            KeyCode::Char('q') => return Ok(Some(Action::Quit)),
            KeyCode::Char('h') | KeyCode::Left => self.list.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.list.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.list.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.list.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.list.select_last(),
            KeyCode::Char(' ') => {
                let idx = self.list.state.selected().unwrap();
                let mut item = self.list.list_items[idx].clone();
                item.selected = !item.selected;
                self.list.list_items[idx] = item;
                return Ok(None);
            }
            KeyCode::Enter => {
                info!("Starting Download");
                let selected = self
                    .list
                    .list_items
                    .iter()
                    .filter(|item| item.selected)
                    .collect_vec();
                self.command_tx
                    .clone()
                    .unwrap()
                    .send(Action::StartDownload(
                        selected
                            .iter()
                            .map(|item| item.attachment.clone())
                            .collect(),
                    ))?;
            }
            KeyCode::Esc => {
                self.command_tx
                    .clone()
                    .unwrap()
                    .send(Action::ToggleDownloadPopup)?;
                return Ok(None);
            }
            _ => {}
        };
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let centered = center(area, Constraint::Percentage(30), Constraint::Percentage(30));
        let items = self
            .list
            .list_items
            .iter()
            .map(ListItem::from)
            .collect_vec();
        let text_btm = "<space> to select, <enter> to download";
        let list_block = Block::new()
            .borders(Borders::ALL)
            .padding(Padding::uniform(1))
            .border_type(BorderType::Rounded)
            .title_top(Line::raw("Attachments").centered().bold())
            .title_bottom(Line::raw(text_btm).centered());
        let list = List::new(items)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(list_block);
        if self.visible {
            frame.render_widget(Clear, centered);
            frame.render_stateful_widget(list, centered, &mut self.list.state);
        }
        Ok(())
    }
    fn get_mode(&self) -> Mode {
        self.mode
    }
}

impl AttachmentList {
    fn select_none(&mut self) {
        self.state.select(None);
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }
    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        self.state.select_last();
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'a> AttachmentListItem {
    fn new(attachment: Attachment) -> Self {
        Self {
            attachment,
            selected: false,
        }
    }
    #[allow(clippy::needless_lifetimes)]
    fn format(&self) -> Line<'a> {
        let selected_span = Span::styled(
            if self.selected { "[x]" } else { "[ ]" }.to_string() + "  ",
            if self.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
                    .add_modifier(Modifier::DIM)
                    .fg(Color::White)
            },
        );
        let text = Span::raw(self.attachment.name.clone());
        let line_style = if self.selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };
        let line = Line::from(vec![selected_span, text]).style(line_style);
        line
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'a> From<&AttachmentListItem> for ListItem<'a> {
    fn from(value: &AttachmentListItem) -> Self {
        ListItem::new(value.format())
    }
}

impl FromIterator<AttachmentListItem> for AttachmentList {
    fn from_iter<I: IntoIterator<Item = AttachmentListItem>>(iter: I) -> Self {
        let items = iter.into_iter().collect();
        let state = ListState::default();
        Self {
            list_items: items,
            state,
        }
    }
}
