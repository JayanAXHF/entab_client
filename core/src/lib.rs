#![allow(non_snake_case, clippy::collapsible_else_if)]

use anyhow::{Context, Ok, Result};
use crossterm::{
    cursor::{Hide, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveUp, RestorePosition, Show},
    event::{self, KeyEvent, KeyModifiers},
    style::{
        Color, Print, PrintStyledContent, ResetColor, SetBackgroundColor, SetForegroundColor,
        Stylize,
    },
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use lazy_static::lazy_static;
use reqwest::{header, Client};
use std::io::{stdout, Write};
use std::{collections::HashMap, fmt};
use std::{env, str::FromStr};
use tl::{parse, ParserOptions};

lazy_static! {
    static ref SESSION_ID_STAT: String = env::var("ENTAB_SESSION_ID")
        .context("Missing ENTAB_SESSION_ID")
        .unwrap();
    static ref REQUEST_VERIFICATION_TOKEN_STAT: String =
        env::var("ENTAB_REQUEST_VERIFICATION_TOKEN")
            .context("Missing ENTAB_REQUEST_VERIFICATION_TOKEN")
            .unwrap();
    static ref ASPXAUTH_STAT: String = env::var("ENTAB_ASPXAUTH")
        .context("Missing ENTAB_ASPXAUTH")
        .unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub id: String,
    pub name: String,
    pub date: String,
    pub type_: String,
    pub s_no: String,
}

pub async fn get_circular() -> Result<Vec<Assignment>> {
    let SESSION_ID: String = SESSION_ID_STAT.clone();
    let REQUEST_VERIFICATION_TOKEN: String = REQUEST_VERIFICATION_TOKEN_STAT.clone();
    let ASPXAUTH: String = ASPXAUTH_STAT.clone();
    let client = Client::new();

    let url = "https://www.lviscampuscare.org/Parent/AssignmentDetailsByAssignmentType";

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        "application/json, text/javascript, */*; q=0.01"
            .parse()
            .unwrap(),
    );
    headers.insert(
        header::CONTENT_TYPE,
        "application/x-www-form-urlencoded; charset=UTF-8"
            .parse()
            .unwrap(),
    );
    headers.insert(
        header::ORIGIN,
        "https://www.lviscampuscare.org".parse().unwrap(),
    );
    headers.insert(
        header::REFERER,
        "https://www.lviscampuscare.org/Parent/Assignment"
            .parse()
            .unwrap(),
    );
    headers.insert(header::USER_AGENT, "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36".parse().unwrap());
    headers.insert("x-requested-with", "XMLHttpRequest".parse().unwrap());

    let cookies = format!("ASP.NET_SessionId={}; chk=enable; __RequestVerificationToken={}; .ASPXAUTH={}; SchoolCode=11674", SESSION_ID, REQUEST_VERIFICATION_TOKEN, ASPXAUTH);
    headers.insert(header::COOKIE, cookies.parse().unwrap());

    let mut form = HashMap::new();
    form.insert("AssignType", "C");
    form.insert("frmDate", "");
    form.insert("toDate", "");
    form.insert("Subject", "");

    let response = client
        .post(url)
        .headers(headers)
        .form(&form)
        .send()
        .await?
        .text()
        .await
        .context("Failed to get response")?;

    let response: serde_json::Value =
        serde_json::from_str(&response).context("Failed to parse response")?;
    let data = response["Data"].as_array().unwrap()[0].as_str().unwrap();
    let parsed_table = parse(data, ParserOptions::default())?;
    let parser = parsed_table.parser();
    let mut rows = vec![];
    parsed_table.nodes().iter().for_each(|row| {
        let tag = row.as_tag();

        if let Some(tag) = tag {
            if tag.name() != "tr" {
                return;
            }
        }
        let subnodes = row.children();
        if let Some(subnodes) = subnodes {
            let subnodes = subnodes.all(parser);
            let mut row = vec![];
            let mut id = String::new();
            for subnode in subnodes {
                let tag = subnode.as_tag();
                if tag.is_some() {
                    let text = subnode.inner_text(parser).to_string();
                    row.push(text.replace(['\r', '\n'], "").trim().to_string());
                    if let Some(a_id) = tag.unwrap().attributes().id() {
                        id = a_id.to_owned().as_utf8_str().to_string();
                    }
                }
            }
            let row = Assignment {
                s_no: row[0].clone(),
                date: row[1].clone(),
                type_: row[2].clone(),
                name: row[3].clone().replace("&#39;", "'").clean_string(),
                id,
            };
            rows.push(row);
        }
    });

    Ok(rows)
}

impl Assignment {
    pub fn field(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.s_no, self.id, self.date, self.type_, self.name
        )
    }
    pub async fn get_details(&self, type_: AssignmentType) -> Result<String> {
        let SESSION_ID: String = SESSION_ID_STAT.clone();
        let REQUEST_VERIFICATION_TOKEN: String = REQUEST_VERIFICATION_TOKEN_STAT.clone();
        let ASPXAUTH: String = ASPXAUTH_STAT.clone();

        let client = Client::new();

        let url = "https://www.lviscampuscare.org/Parent/GetAssignemtDetails";

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            "application/json, text/javascript, */*; q=0.01"
                .parse()
                .unwrap(),
        );
        headers.insert(
            header::CONTENT_TYPE,
            "application/x-www-form-urlencoded; charset=UTF-8"
                .parse()
                .unwrap(),
        );
        headers.insert(
            header::ORIGIN,
            "https://www.lviscampuscare.org".parse().unwrap(),
        );
        headers.insert(
            header::REFERER,
            "https://www.lviscampuscare.org/Parent/Assignment"
                .parse()
                .unwrap(),
        );
        headers.insert(header::USER_AGENT, "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36".parse().unwrap());
        headers.insert("x-requested-with", "XMLHttpRequest".parse().unwrap());

        let cookies = format!("ASP.NET_SessionId={}; chk=enable; __RequestVerificationToken={}; .ASPXAUTH={}; SchoolCode=11674", SESSION_ID, REQUEST_VERIFICATION_TOKEN, ASPXAUTH);
        headers.insert(header::COOKIE, cookies.parse().unwrap());

        let mut form = HashMap::new();
        let type_ = type_.to_string();
        form.insert("frmDate", "");
        form.insert("AssignType", &type_);
        form.insert("toDate", "");
        form.insert("Subject", "0");
        form.insert("AssigID", self.id.as_str());

        let response = client
            .post(url)
            .headers(headers)
            .form(&form)
            .send()
            .await?
            .text()
            .await
            .context("Failed to get response")?;

        let response: serde_json::Value =
            serde_json::from_str(&response).context("Failed to parse response")?;
        let data = response["Data"].as_array().unwrap()[0]["Assignment"]
            .as_str()
            .unwrap();
        let parsed_table = parse(data, ParserOptions::default())?;
        let mut out = String::new();
        let parser = parsed_table.parser();
        parsed_table.children().iter().for_each(|tag| {
            let text = tag.get(parser).unwrap().inner_text(parser).to_string();
            out.push_str(&text);
            out.push_str("\r\n");
        });

        let attachments = response["Data"].as_array().unwrap()[3].as_array().unwrap();
        let links = attachments.iter().map(|attachment| {
            let filename = attachment["Attachment"].as_str().unwrap();
            let url = format!("https://www.lviscampuscare.org/Assignment/{}", filename);
            Link::new(filename.to_string(), url)
        });
        out.push_str("\r\n");
        links.for_each(|link| {
            out.push_str(&link.to_string());
            out.push('\t');
        });
        let out = out.clean_string();

        Ok(out)
    }
}

pub struct App {
    assignments: Vec<Assignment>,
    selected_assignment: Option<Assignment>,
    assignment_type: AssignmentType,
    window_start: usize,
    window_size: usize,
    mode: Modes,
    assignment_filter: String,
}

impl App {
    pub async fn new(type_: AssignmentType) -> Self {
        let assignments = match type_ {
            AssignmentType::Circular => get_circular().await.unwrap(),
            AssignmentType::Homework => homework::get_hw().await.unwrap(),
        };
        let assignments = assignments
            .into_iter()
            .map(|a| {
                let mut number = a.s_no;
                if number.len() == 1 {
                    number.insert(0, '0');
                }
                Assignment { s_no: number, ..a }
            })
            .collect::<Vec<_>>();
        let selected_assignment = None;
        let (_, window_size) = terminal::size().unwrap();
        Self {
            assignments,
            selected_assignment,
            assignment_type: type_,
            window_start: 0,
            window_size: window_size as usize - 2,
            mode: Modes::ViewingList,
            assignment_filter: String::new(),
        }
    }
    pub fn change_mode(&mut self, mode: Modes) {
        self.mode = mode;
    }
    pub fn cycle_modes(&mut self) {
        match self.mode {
            Modes::ViewingList => self.mode = Modes::Filtering,
            Modes::Filtering => self.mode = Modes::ViewingList,
        }
    }
    pub fn print_table(&self) -> Result<()> {
        stdout().queue(Clear(ClearType::All))?;
        stdout().queue(MoveTo(0, 1))?;
        let rows = self.assignments.clone();

        let rows = rows
            .iter()
            .filter(|a| {
                let str = format!("{} {} {}", a.name, a.type_, a.date);
                str.to_lowercase()
                    .contains(&self.assignment_filter.to_lowercase())
            })
            .cloned()
            .collect::<Vec<_>>()
            .clone();
        for row in rows
            .into_iter()
            .skip(self.window_start)
            .take(self.window_size)
        {
            let mut text = row.field();
            if let Some(selected_assignment) = self.get_selected_assignment() {
                if selected_assignment.s_no == row.s_no {
                    text.insert_str(0, "\x1b[96m> \x1b[0m\x1b[30;107m");
                    stdout().queue(SetBackgroundColor(Color::White))?;
                    stdout().queue(SetForegroundColor(Color::Black))?;
                } else {
                    text.insert_str(0, "  ");
                }
            }
            stdout().queue(Print(text))?;
            stdout().queue(ResetColor)?;
            stdout().queue(Print("\n"))?;
            stdout().queue(MoveToColumn(0))?;
        }

        stdout().queue(RestorePosition)?;
        stdout().flush()?;
        Ok(())
    }
    pub fn select_assignment(&mut self, assignment: Assignment) {
        self.selected_assignment = Some(assignment);
    }
    pub fn get_selected_assignment(&self) -> Option<Assignment> {
        self.selected_assignment.clone()
    }
    fn adjust_window(&mut self, selected_index: usize) {
        if selected_index < self.window_start {
            self.window_start = selected_index;
        } else if selected_index >= self.window_start + self.window_size {
            self.window_start = selected_index + 1 - self.window_size;
        }
    }

    async fn handle_list_navigation(&mut self, key: KeyEvent) -> Result<()> {
        stdout().execute(Hide)?;
        let assignments = self.assignments.clone();
        let assignments = assignments
            .iter()
            .filter(|a| {
                let str = format!("{} {} {}", a.name, a.type_, a.date);
                str.to_lowercase()
                    .contains(&self.assignment_filter.to_lowercase())
            })
            .cloned()
            .collect::<Vec<_>>()
            .clone();

        match key {
            KeyEvent {
                code: event::KeyCode::Char('j'),
                ..
            } => {
                if let Some(selected_assignment) = self.get_selected_assignment() {
                    let index = assignments
                        .iter()
                        .position(|a| a == &selected_assignment)
                        .unwrap_or(0)
                        + 1;
                    if index == assignments.len() {
                        self.select_assignment(assignments[0].clone());
                        self.window_start = 0;
                    }
                    if let Some(next_assignment) = assignments.get(index) {
                        self.select_assignment(next_assignment.clone());
                        self.adjust_window(index);
                    }
                } else {
                    self.select_assignment(self.assignments[0].clone());
                    self.window_start = 0;
                }
            }
            KeyEvent {
                code: event::KeyCode::Char('k'),
                ..
            } => {
                if let Some(selected_assignment) = self.get_selected_assignment() {
                    let index = assignments
                        .iter()
                        .position(|a| a == &selected_assignment)
                        .unwrap_or(0)
                        + 1;

                    if index == 1 {
                        self.select_assignment(assignments[assignments.len() - 1].clone());
                        self.window_start = assignments.len().saturating_sub(self.window_size);
                    } else {
                        if let Some(next_assignment) = assignments.get(index - 2) {
                            self.select_assignment(next_assignment.clone());
                            self.adjust_window(index - 2);
                        }
                    }
                } else {
                    self.select_assignment(self.assignments[0].clone());
                    self.window_start = 0;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_typing(&mut self, key: KeyEvent) -> Result<()> {
        stdout().execute(Show)?;
        match key.code {
            event::KeyCode::Left => {
                stdout().execute(MoveLeft(1))?;
            }
            event::KeyCode::Right => {
                stdout().execute(MoveRight(1))?;
            }
            event::KeyCode::Backspace => {
                self.assignment_filter.pop();
                self.print_table()?;
            }
            event::KeyCode::Char(c) => {
                if c.is_ascii() {
                    self.assignment_filter.push(c);
                    self.print_table()?;
                }
            }
            _ => {}
        }

        Ok(())
    }
    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(MoveTo(0, 0))?;
        self.print_table()?;
        stdout().execute(Hide)?;
        loop {
            stdout().execute(MoveUp(self.assignments.len() as u16))?;
            if let Modes::ViewingList = self.mode {
                stdout().execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                self.print_table()?;
                stdout().execute(MoveUp(self.assignments.len() as u16))?;
            }
            stdout().execute(MoveTo(0, 0))?;
            stdout().execute(PrintStyledContent("? Filter: ".with(Color::Green)))?;
            stdout().execute(Print(self.assignment_filter.clone()))?;

            let event = event::read().unwrap();
            if let event::Event::Key(key) = event {
                match self.mode {
                    Modes::ViewingList => self.handle_list_navigation(key).await?,
                    Modes::Filtering => self.handle_typing(key).await?,
                };
                match key {
                    KeyEvent {
                        code: event::KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        break;
                    }
                    KeyEvent {
                        code: event::KeyCode::Tab,
                        ..
                    } => {
                        self.cycle_modes();
                    }
                    KeyEvent {
                        code: event::KeyCode::Enter,
                        ..
                    } => {
                        if let Some(selected_assignment) = self.get_selected_assignment() {
                            let details = selected_assignment
                                .get_details(self.assignment_type.clone())
                                .await
                                .unwrap();
                            stdout().execute(Clear(ClearType::All))?;
                            stdout().execute(Print("\r\n"))?;
                            stdout().execute(Print(details))?;
                            break;
                        }
                    }

                    _ => {}
                }
            }
        }

        stdout().execute(Show)?;
        disable_raw_mode()?;
        Ok(())
    }
}

pub struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        stdout().execute(Show).unwrap();
    }
}

pub trait CleanString {
    fn clean_string(self) -> Self;
}

impl CleanString for String {
    fn clean_string(self) -> String {
        let replace_map = [
            ("&nbsp;", " "),
            ("&quot;", "\""),
            ("&amp;", "&"),
            ("&lt;", "<"),
            ("&gt;", ">"),
            ("&#39;", "'"),
        ];
        let mut out = self;
        for (from, to) in replace_map.iter() {
            out = out.replace(from, to);
        }
        out.trim().to_string()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Link {
    pub text: String,
    pub url: String,
}

impl Link {
    pub fn new(text: String, url: String) -> Self {
        Self { text, url }
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            self.url, self.text
        )
    }
}

#[derive(Clone, Debug)]
pub enum AssignmentType {
    Circular,
    Homework,
}

impl fmt::Display for AssignmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssignmentType::Circular => write!(f, "C"),
            AssignmentType::Homework => write!(f, "H"),
        }
    }
}

impl FromStr for AssignmentType {
    type Err = String; // Return an error message instead of just a unit type.

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "H" => std::result::Result::Ok(AssignmentType::Homework),
            "C" => std::result::Result::Ok(AssignmentType::Circular),
            _ => Err(format!("Invalid assignment type: '{}'", s)),
        }
    }
}
pub mod homework {
    use super::*;
    pub async fn get_hw() -> Result<Vec<Assignment>> {
        let SESSION_ID: String = SESSION_ID_STAT.clone();
        let REQUEST_VERIFICATION_TOKEN: String = REQUEST_VERIFICATION_TOKEN_STAT.clone();
        let ASPXAUTH: String = ASPXAUTH_STAT.clone();
        let client = Client::new();

        let url = "https://www.lviscampuscare.org/Parent/AssignmentDetailsByAssignmentType";

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            "application/json, text/javascript, */*; q=0.01"
                .parse()
                .unwrap(),
        );
        headers.insert(
            header::CONTENT_TYPE,
            "application/x-www-form-urlencoded; charset=UTF-8"
                .parse()
                .unwrap(),
        );
        headers.insert(
            header::ORIGIN,
            "https://www.lviscampuscare.org".parse().unwrap(),
        );
        headers.insert(
            header::REFERER,
            "https://www.lviscampuscare.org/Parent/Assignment"
                .parse()
                .unwrap(),
        );
        headers.insert(header::USER_AGENT, "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36".parse().unwrap());
        headers.insert("x-requested-with", "XMLHttpRequest".parse().unwrap());

        let cookies = format!("ASP.NET_SessionId={}; chk=enable; __RequestVerificationToken={}; .ASPXAUTH={}; SchoolCode=11674", SESSION_ID, REQUEST_VERIFICATION_TOKEN, ASPXAUTH);
        headers.insert(header::COOKIE, cookies.parse().unwrap());

        let mut form = HashMap::new();
        form.insert("AssignType", "H");
        form.insert("frmDate", "");
        form.insert("toDate", "");
        form.insert("Subject", "");

        let response = client
            .post(url)
            .headers(headers)
            .form(&form)
            .send()
            .await?
            .text()
            .await
            .context("Failed to get response")?;

        let response: serde_json::Value =
            serde_json::from_str(&response).context("Failed to parse response")?;
        let data = response["Data"].as_array().unwrap()[0].as_str().unwrap();
        let parsed_table = parse(data, ParserOptions::default())?;
        let parser = parsed_table.parser();
        let mut rows = vec![];
        parsed_table.nodes().iter().for_each(|row| {
            let tag = row.as_tag();

            if let Some(tag) = tag {
                if tag.name() != "tr" {
                    return;
                }
            }
            let subnodes = row.children();
            if let Some(subnodes) = subnodes {
                let subnodes = subnodes.all(parser);
                let mut row = vec![];
                let mut id = String::new();
                for subnode in subnodes {
                    let tag = subnode.as_tag();
                    if tag.is_some() {
                        let text = subnode.inner_text(parser).to_string();
                        row.push(text.replace(['\r', '\n'], "").trim().to_string());
                        if let Some(a_id) = tag.unwrap().attributes().id() {
                            id = a_id.to_owned().as_utf8_str().to_string();
                        }
                    }
                }
                let row = Assignment {
                    s_no: row[0].clone(),
                    date: row[1].clone(),
                    type_: row[2].clone(),
                    name: row[3].clone().clean_string(),
                    id,
                };
                rows.push(row);
            }
        });

        Ok(rows)
    }
}

#[derive(Clone, Debug)]
pub enum Modes {
    ViewingList,
    Filtering,
}
