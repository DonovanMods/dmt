use quick_xml::events::{BytesStart, BytesText, Event};
use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    io::Write,
    str::from_utf8,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CsvInstruction {
    Add(char),
    Remove(char),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InstructionSet {
    pub attribute: Option<Vec<u8>>,
    pub csv_op: Option<CsvInstruction>,
    pub values: Vec<Event<'static>>,
    pub xpath: Vec<u8>,
}

impl InstructionSet {
    pub fn new() -> Self {
        Self::default()
    }

    fn values_to_strings(&self) -> Vec<String> {
        self.values
            .iter()
            .map(|e| from_utf8(e.to_vec().as_slice()).unwrap_or_default().to_owned())
            .collect()
    }
}

// Modlet types that require additional lines to be added after the Start event
pub const COLLECTION_COMMANDS: [&str; 3] = ["append", "insert_after", "insert_before"];
// Modlet types that require additional TEXT lines added
pub const TEXT_COMMANDS: [&str; 3] = ["csv", "set", "set_attribute"];
// Modlet types that are empty tags
pub const EMPTY_COMMANDS: [&str; 2] = ["remove", "remove_attribute"];

/// Represents a modlet command instruction
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Append(InstructionSet),
    Comment(Cow<'static, str>),
    Csv(InstructionSet),
    InsertAfter(InstructionSet),
    InsertBefore(InstructionSet),
    NoOp,
    Remove(InstructionSet),
    RemoveAttribute(InstructionSet),
    Set(InstructionSet),
    SetAttribute(InstructionSet),
    StartTag(Option<String>),
    Unknown,
}

impl AsRef<str> for Command {
    fn as_ref(&self) -> &str {
        match self {
            Command::Append(_) => "append",
            Command::Comment(_) => "comment",
            Command::Csv(_) => "csv",
            Command::InsertAfter(_) => "insert_after",
            Command::InsertBefore(_) => "insert_before",
            Command::NoOp => "no_op",
            Command::Remove(_) => "remove",
            Command::RemoveAttribute(_) => "remove_attribute",
            Command::Set(_) => "set",
            Command::SetAttribute(_) => "set_attribute",
            Command::StartTag(_) => "start_tag",
            Command::Unknown => "unknown",
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Append(_) => write!(f, "append"),
            Command::Comment(_) => write!(f, "comment"),
            Command::Csv(_) => write!(f, "csv"),
            Command::InsertAfter(_) => write!(f, "insert_after"),
            Command::InsertBefore(_) => write!(f, "insert_before"),
            Command::NoOp => write!(f, "no_op"),
            Command::Remove(_) => write!(f, "remove"),
            Command::RemoveAttribute(_) => write!(f, "remove_attribute"),
            Command::Set(_) => write!(f, "set"),
            Command::SetAttribute(_) => write!(f, "set_attribute"),
            Command::StartTag(_) => write!(f, "start_tag"),
            Command::Unknown => write!(f, "unknown"),
        }
    }
}

impl Command {
    pub fn from_str(cmd: &str) -> Self {
        match cmd {
            "append" => Command::Append(InstructionSet::new()),
            "comment" => Command::Comment(Cow::Owned(String::new())),
            "csv" => Command::Csv(InstructionSet::new()),
            "insert_after" => Command::InsertAfter(InstructionSet::new()),
            "insert_before" => Command::InsertBefore(InstructionSet::new()),
            "no_op" => Command::NoOp,
            "remove" => Command::Remove(InstructionSet::new()),
            "remove_attribute" => Command::RemoveAttribute(InstructionSet::new()),
            "set" => Command::Set(InstructionSet::new()),
            "set_attribute" => Command::SetAttribute(InstructionSet::new()),
            "start_tag" => Command::StartTag(None),
            _ => Command::Unknown,
        }
    }

    pub fn set(self, instruction_set: InstructionSet) -> Self {
        match self {
            Command::Append(_) => Self::Append(instruction_set),
            Command::Comment(_) => Self::Comment(Cow::Owned(instruction_set.values_to_strings().join(","))),
            Command::Csv(_) => Self::Csv(instruction_set),
            Command::InsertAfter(_) => Self::InsertAfter(instruction_set),
            Command::InsertBefore(_) => Self::InsertBefore(instruction_set),
            Command::NoOp => Self::NoOp,
            Command::Remove(_) => Self::Remove(instruction_set),
            Command::RemoveAttribute(_) => Self::RemoveAttribute(instruction_set),
            Command::Set(_) => Self::Set(instruction_set),
            Command::SetAttribute(_) => Self::SetAttribute(instruction_set),
            Command::StartTag(_) => Self::StartTag(None),
            Command::Unknown => Self::Unknown,
        }
    }

    pub fn write(&self, writer: &mut quick_xml::Writer<impl Write>) -> eyre::Result<()> {
        match self {
            // TODO: This isn't working, values need to be parsed into XML structs
            Command::Append(is) => {
                writer
                    .create_element("append")
                    .with_attribute((b"xpath".as_ref(), is.xpath.as_slice()))
                    .write_inner_content(move |writer| {
                        for event in &is.values {
                            writer.write_event(event)?;
                        }
                        Ok::<(), eyre::Error>(())
                    })?;
            }
            Command::Comment(comment) => {
                let comment = BytesText::from_escaped(comment.clone());
                writer.write_event(Event::Comment(comment))?
            }
            Command::Csv(_) => (),
            Command::InsertAfter(_) => (),
            Command::InsertBefore(_) => (),
            Command::Remove(_) => (),
            Command::RemoveAttribute(_) => (),
            Command::Set(is) => {
                writer
                    .create_element("set")
                    .with_attribute((b"xpath".as_ref(), is.xpath.as_ref()))
                    .write_text_content(BytesText::new(is.values_to_strings().join(",").as_ref()))?;
            }
            Command::SetAttribute(_) => (),
            Command::StartTag(_) => (),
            _ => (),
        }

        Ok(())
    }
}
