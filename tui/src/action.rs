use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    Mode(crate::app::Mode),
    AssignmentType(client_core::AssignmentType),
    AssignmentDetails(Option<String>),
    ToggleDownloadPopup,
    Attachments(Vec<client_core::Attachment>),
    Assignment(client_core::Assignment),
}
