use client_core::{Assignment, get_circular, homework};
use color_eyre::Result;
use crossterm::event::KeyCode;
use futures::executor::block_on;
use itertools::Itertools;
use ratatui::widgets::List as ListWidget;
use ratatui::{prelude::*, widgets::*};
use std::io::Write;
use style::palette::tailwind::SLATE;
use tabwriter::TabWriter;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tui_input::{Input, backend::crossterm::EventHandler};

use super::Component;
use crate::{action::Action, app::Mode, config::Config};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(Default)]
pub struct List {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list: AssignmentList,
    mode: Mode,
    enabled: bool,
    state: State,
    input: Input,
    assignment_type: client_core::AssignmentType,
    assignments: Vec<Assignment>,
}

impl List {
    pub fn new() -> Self {
        Self {
            mode: Mode::ListScreen,
            list: AssignmentList::default(),
            state: State::Normal,
            ..Default::default()
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
enum State {
    #[default]
    Normal,
    Search,
}

#[derive(Debug, Clone, Default)]
pub struct AssignmentList {
    list_items: Vec<AssignmentListItem>,
    filtered_items: Vec<AssignmentListItem>,
    state: ListState,
}

#[derive(Debug, Clone, Default)]
pub struct AssignmentListItem {
    assignment: Assignment,
}

impl Component for List {
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

            Action::AssignmentType(type_) => {
                self.assignment_type = type_;
                let assignments = match type_ {
                    client_core::AssignmentType::Circular => block_on(get_circular()).unwrap(),
                    client_core::AssignmentType::Homework => block_on(homework::get_hw()).unwrap(),
                };
                self.assignments = assignments.clone();
                let assignment_list_items = assignments
                    .into_iter()
                    .map(AssignmentListItem::new)
                    .collect::<Vec<AssignmentListItem>>();
                let assignment_list = AssignmentList::from_iter(assignment_list_items);
                self.list = assignment_list;
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        if !self.enabled {
            return Ok(None);
        }
        if self.state == State::Search {
            match key.code {
                KeyCode::Tab | KeyCode::Esc => self.toggle_state(),
                KeyCode::Enter => {}
                _ => {
                    self.input.handle_event(&crossterm::event::Event::Key(key));
                    let val = self.input.value();
                    let filtered_items = self
                        .list
                        .list_items
                        .iter()
                        .filter(|item| {
                            let a = &item.assignment;
                            let str = format!("{} {} {}", a.name, a.type_, a.date);
                            str.to_lowercase().contains(&val.to_lowercase())
                        })
                        .cloned()
                        .collect();
                    self.list.filtered_items = filtered_items;
                    self.list.state.select_first();
                }
            }
            return Ok(None);
        }
        info!("Got key: {key:?}");
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.list.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.list.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.list.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.list.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.list.select_last(),
            KeyCode::Esc => {
                self.command_tx.clone().unwrap().send(Action::ClearScreen)?;
                return Ok(Some(Action::Mode(Mode::Home)));
            }
            KeyCode::Enter => {
                let selected_index = self.list.state.selected().unwrap();
                let selected_assignment = if self.list.filtered_items.is_empty() {
                    self.list.list_items[selected_index].assignment.clone()
                } else {
                    self.list.filtered_items[selected_index].assignment.clone()
                };
                let details = block_on(selected_assignment.get_details(self.assignment_type))
                    .expect("Unable to get assignment details");
                self.command_tx.clone().unwrap().send(Action::ClearScreen)?;
                self.command_tx
                    .clone()
                    .unwrap()
                    .send(Action::Assignment(selected_assignment))?;
                self.command_tx
                    .clone()
                    .unwrap()
                    .send(Action::AssignmentDetails(Some(details)))?;
                self.command_tx
                    .clone()
                    .unwrap()
                    .send(Action::Mode(Mode::CurrentAssignmentScreen))?;
            }
            KeyCode::Char('q') => return Ok(Some(Action::Quit)),
            KeyCode::Char('/') => self.toggle_state(),
            _ => {}
        };
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        //   let items: Vec<ListItem> = if self.list.filtered_items.is_empty() {
        //       self.list.list_items.iter().map(ListItem::from).collect()
        //   } else {
        //       self.list
        //           .filtered_items
        //           .iter()
        //           .map(ListItem::from)
        //           .collect()
        //   };
        let mut tw = TabWriter::new(vec![]);
        if self.list.filtered_items.is_empty() {
            write!(
                tw,
                "{}",
                self.list
                    .list_items
                    .iter()
                    .map(AssignmentListItem::format)
                    .join("\n")
            )
            .unwrap();
        } else {
            write!(
                tw,
                "{}",
                self.list
                    .filtered_items
                    .iter()
                    .map(AssignmentListItem::format)
                    .join("\n")
            )
            .unwrap();
        }
        let written = String::from_utf8(tw.into_inner().unwrap()).unwrap();
        let items = written.lines().map(|line| ListItem::new(line.to_string()));

        let list_style = match self.state {
            State::Normal => Color::Yellow.into(),
            State::Search => Style::default(),
        };
        let list = ListWidget::new(items)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .padding(Padding::uniform(1))
                    .border_type(BorderType::Rounded)
                    .border_style(list_style)
                    .title_top(Line::raw("Assignments").centered().bold())
                    .title_bottom(
                        Line::raw("Press j/k or Up/Down to move, <Enter> to select").centered(),
                    )
                    .title_bottom(Line::raw("Press `q` to quit, <Esc> to go back").right_aligned()),
            );
        let [top, center] =
            Layout::vertical([Constraint::Min(3), Constraint::Percentage(100)]).areas(area);

        let style = match self.state {
            State::Normal => Style::default(),
            State::Search => Color::Yellow.into(),
        };
        let input_title = match self.state {
            State::Normal => "Input (Press `/` to search)".to_string(),
            State::Search => "Input (Press `Esc`/`Tab` to exit)".to_string(),
        };

        let input = Paragraph::new(self.input.value()).style(style).block(
            Block::bordered()
                .title(input_title)
                .border_type(BorderType::Rounded),
        );
        frame.render_widget(input, top);
        frame.render_stateful_widget(list, center, &mut self.list.state);
        Ok(())
    }
    fn get_mode(&self) -> crate::app::Mode {
        self.mode
    }
}

impl AssignmentList {
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
impl FromIterator<AssignmentListItem> for AssignmentList {
    fn from_iter<I: IntoIterator<Item = AssignmentListItem>>(iter: I) -> Self {
        let items = iter.into_iter().collect();
        let state = ListState::default();
        Self {
            filtered_items: Vec::new(),
            list_items: items,
            state,
        }
    }
}

impl List {
    pub fn toggle_state(&mut self) {
        self.state = match self.state {
            State::Normal => State::Search,
            State::Search => State::Normal,
        };
    }
}

impl AssignmentListItem {
    fn format(&self) -> String {
        info!("Got assignment name: {}", self.assignment.name);
        format!(
            "{}\t{}\t{}\t{}\t{}",
            self.assignment.s_no,
            self.assignment.id,
            self.assignment.date,
            self.assignment.type_,
            self.assignment.name
        )
    }
    fn new(assignment: Assignment) -> Self {
        Self { assignment }
    }
}

impl From<&AssignmentListItem> for ListItem<'_> {
    fn from(value: &AssignmentListItem) -> Self {
        let val = value.format();
        ListItem::new(val)
    }
}
