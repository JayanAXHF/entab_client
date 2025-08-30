use client_core::AssignmentType;
use color_eyre::Result;
use crossterm::event::KeyCode;
use itertools::Itertools;
use layout::Flex;
use ratatui::{prelude::*, widgets::*};
use strum::IntoEnumIterator;

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
pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list: ModeList,
    mode: Mode,
    enabled: bool,
}

impl Home {
    pub fn new() -> Self {
        Home {
            list: ModeList::from_iter(vec![AssignmentType::Circular, AssignmentType::Homework]),
            mode: Mode::Home,
            enabled: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModeList {
    list_items: Vec<ModeListItem>,
    state: ListState,
}

#[derive(Debug, Clone, Default)]
pub struct ModeListItem {
    mode: AssignmentType,
}

impl ModeListItem {
    fn new(mode: AssignmentType) -> Self {
        Self { mode }
    }
}

impl ModeList {
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

impl Component for Home {
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
            Action::Render => {
                // add any logic here that should run on every render
            }
            Action::Mode(mode) => {
                self.enabled = mode == self.mode;
            }
            _ => {}
        }
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        info!("Got key: {key:?}");

        if !self.enabled {
            return Ok(None);
        }
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.list.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.list.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.list.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.list.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.list.select_last(),
            KeyCode::Enter => {
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::AssignmentType(
                        self.list.list_items[self.list.state.selected().unwrap_or(0)].mode,
                    ))?;
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::Mode(Mode::ListScreen))?;
                return Ok(Some(Action::Mode(Mode::ListScreen)));
            }
            KeyCode::Char('q') => return Ok(Some(Action::Quit)),
            _ => {}
        };
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let items: Vec<ListItem> = self.list.list_items.iter().map(ListItem::from).collect();
        let list = List::new(items)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always)
            .scroll_padding(5)
            .block(
                Block::new()
                    .padding(Padding::uniform(1))
                    .title_top(Line::raw("Modes").centered().bold()),
            );

        let center_area = center(
            area,
            Constraint::Percentage(15),
            Constraint::Length(7), // top and bottom border + content
        );
        frame.render_stateful_widget(list, center_area, &mut self.list.state);

        Ok(())
    }
    fn get_mode(&self) -> Mode {
        self.mode
    }
}

impl From<&ModeListItem> for ListItem<'_> {
    fn from(value: &ModeListItem) -> Self {
        let val = match value.mode {
            AssignmentType::Circular => "Circular",
            AssignmentType::Homework => "Homework",
        };
        ListItem::new(val)
    }
}

impl FromIterator<AssignmentType> for ModeList {
    fn from_iter<I: IntoIterator<Item = AssignmentType>>(iter: I) -> Self {
        let items = iter.into_iter().map(ModeListItem::new).collect();
        let state = ListState::default();
        Self {
            list_items: items,
            state,
        }
    }
}
