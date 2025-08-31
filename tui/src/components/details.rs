use std::cmp::max;

use color_eyre::Result;
use crossterm::event::KeyCode;
use layout::Flex;
use ratatui::{prelude::*, widgets::*};

use style::palette::tailwind::SLATE;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tui_scrollview::{ScrollView, ScrollViewState};

use super::Component;
use crate::{action::Action, app::Mode, config::Config};

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

#[derive(Default)]
pub struct Details {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
    enabled: bool,
    current_assignment: Option<String>,
    scrollview_state: ScrollViewState,
}

impl Details {
    pub fn new() -> Self {
        Details {
            mode: Mode::CurrentAssignmentScreen,
            enabled: true,
            ..Default::default()
        }
    }
}

impl Component for Details {
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
            Action::AssignmentDetails(assignment) => {
                self.current_assignment = assignment;
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
            KeyCode::Char('q') => return Ok(Some(Action::Quit)),
            KeyCode::Char('j') | KeyCode::Down => self.scrollview_state.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.scrollview_state.scroll_up(),
            KeyCode::Char('f') | KeyCode::PageDown => self.scrollview_state.scroll_page_down(),
            KeyCode::Char('b') | KeyCode::PageUp => self.scrollview_state.scroll_page_up(),
            KeyCode::Esc => {
                self.command_tx.clone().unwrap().send(Action::ClearScreen)?;
                return Ok(Some(Action::Mode(Mode::ListScreen)));
            }
            _ => {}
        };
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let centered = center(area, Constraint::Percentage(50), Constraint::Percentage(50));
        let assignment = self.current_assignment.clone().unwrap_or_default();
        let size = Size::new(
            centered.width,
            max(assignment.lines().count() as u16, centered.height),
        );
        let mut scrollview = ScrollView::new(size)
            .horizontal_scrollbar_visibility(tui_scrollview::ScrollbarVisibility::Never);
        let para = Paragraph::new(assignment)
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Assignment Details")
                    .padding(Padding::uniform(1))
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(SLATE.c500)),
            )
            .wrap(Wrap { trim: true });
        scrollview.render_widget(para, scrollview.area());
        frame.render_stateful_widget(scrollview, centered, &mut self.scrollview_state);
        Ok(())
    }
    fn get_mode(&self) -> Mode {
        self.mode
    }
}
