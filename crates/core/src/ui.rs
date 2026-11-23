//! What the page draws. The core describes a screen as panels of rows, fields
//! and actions; the page builds the DOM from this without knowing what any of
//! it means, and sends an action's name and value back through `act`.

use serde::Serialize;
use serde_json::Value;

use crate::data::Request;

/// One button: the action name and value the page sends back through `act`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Action {
    pub name: String,
    pub value: String,
    pub label: String,
    pub enabled: bool,
}

impl Action {
    pub fn new(name: &str, value: &str, label: &str) -> Action {
        Action {
            name: name.to_string(),
            value: value.to_string(),
            label: label.to_string(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

/// One input of a form. `kind` is `text`, `number`, `date`, `password` or `select`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Field {
    pub name: String,
    pub label: String,
    pub kind: String,
    pub value: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<Choice>,
    /// The longest text the field takes, in characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
}

impl Field {
    pub fn new(name: &str, label: &str, kind: &str, value: &str) -> Field {
        Field {
            name: name.to_string(),
            label: label.to_string(),
            kind: kind.to_string(),
            value: value.to_string(),
            choices: Vec::new(),
            max: None,
        }
    }

    pub fn with_max(mut self, max: u32) -> Field {
        self.max = Some(max);
        self
    }

    pub fn with_choices(mut self, choices: Vec<Choice>) -> Field {
        self.choices = choices;
        self
    }
}

/// A line of a list: a block of a session, a day of a week, a max.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Row {
    pub id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub detail: String,
    /// A word that says what kind of thing the row is (a session kind, "Voice note").
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tag: String,
    /// A word about the row's state: "Moved from Tue", "Today".
    #[serde(skip_serializing_if = "String::is_empty")]
    pub marker: String,
    /// The coach's note, word for word.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub note: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<Row>,
    /// Whether the rows under this one start shown.
    pub open: bool,
    /// The lift category the row belongs to (`power`, `strength`, ...); the page colours it and its tag says it in words.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub category: String,
    /// The categories a day holds, as (key, label) pairs, for the week screen.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub chips: Vec<Choice>,
    /// The athlete whose picture goes with this row, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<u64>,
    /// An athlete's name when the row is a person to tap (the page draws the initial without a picture).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub initial: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
}

impl Row {
    pub fn new(id: &str, label: &str) -> Row {
        Row {
            id: id.to_string(),
            label: label.to_string(),
            ..Row::default()
        }
    }
}

/// A block of the screen. `kind` is `text`, `list`, `session`, `form` or `recorder`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Panel {
    pub id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub heading: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub marker: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<Row>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Field>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
    /// For a recorder: the longest recording, in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// For a recorder: the lifts a note can go on.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lifts: Vec<Choice>,
    /// Numbers the page's drawing needs (the hero's two jump heights, in inches).
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub data: std::collections::BTreeMap<String, String>,
}

impl Panel {
    pub fn new(id: &str, kind: &str) -> Panel {
        Panel {
            id: id.to_string(),
            kind: kind.to_string(),
            heading: String::new(),
            text: String::new(),
            marker: String::new(),
            rows: Vec::new(),
            fields: Vec::new(),
            actions: Vec::new(),
            limit: None,
            lifts: Vec::new(),
            data: std::collections::BTreeMap::new(),
        }
    }

    pub fn heading(mut self, heading: &str) -> Panel {
        self.heading = heading.to_string();
        self
    }

    pub fn text(mut self, text: &str) -> Panel {
        self.text = text.to_string();
        self
    }
}

/// A term and its definition, for the page's hover hints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hint {
    pub term: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Header {
    /// The athlete's name, or empty before one is chosen.
    pub who: String,
    pub coach: bool,
    /// The signed-in athlete, when they have a picture.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<u64>,
    /// "Offline, as of …" when the phone cannot reach the database.
    pub offline: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NavItem {
    pub label: String,
    pub name: String,
    pub value: String,
    pub current: bool,
}

/// Everything the page draws, and what it must do next.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ui<S: Serialize> {
    pub screen: String,
    pub title: String,
    pub header: Header,
    pub nav: Vec<NavItem>,
    pub panels: Vec<Panel>,
    pub hints: Vec<Hint>,
    /// Calls to make to the database, in order, then refetch.
    pub requests: Vec<Request>,
    /// Text for the phone to keep under `choice`; empty when there is none.
    pub keep: String,
    /// The sign-in the page sends with the calls it makes, when one was just chosen.
    pub code: String,
    pub message: String,
    /// The page's own words (the `page_` strings), by key.
    pub labels: std::collections::BTreeMap<String, String>,
    pub state: S,
}

/// A JSON value for a request's arguments, made from pairs.
pub fn args(pairs: Vec<(&str, Value)>) -> Value {
    let mut map = serde_json::Map::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v);
    }
    Value::Object(map)
}
