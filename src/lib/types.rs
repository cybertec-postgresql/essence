use std::fmt;
use std::io;

pub const EVENT_TRIGGER_FOLDER: &str = "event_triggers";
pub const EXTENSION_FOLDER: &str = "extensions";
pub const ROLE_FOLDER: &str = "roles";
pub const SCHEMA_FOLDER: &str = "schemas";

pub const FUNCTION_FOLDER: &str = "functions";
pub const TABLE_FOLDER: &str = "tables";
pub const TYPE_FOLDER: &str = "types";
pub const VIEW_FOLDER: &str = "views";

#[derive(PartialEq)]
pub enum Mode {
    Preamble,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Acl,
    Aggregate,
    Comment,
    CheckConstraint,
    Constraint,
    Database,
    Default,
    DefaultAcl,
    Domain,
    EventTrigger,
    Extension,
    FkConstraint,
    ForeignTable,
    Function,
    Index,
    MaterializedView,
    MaterializedViewData,
    Policy,
    RowSecurity,
    Rule,
    Schema,
    Sequence,
    SequenceOwnedBy,
    SequenceSet,
    Table,
    TableData,
    Trigger,
    Type,
    View,
}

#[derive(Debug)]
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub schema: Option<String>,
    pub owner: String,
}

impl Default for Entity {
    fn default() -> Entity {
        Entity {
            name: "".to_string(),
            entity_type: EntityType::Acl,
            schema: None,
            owner: "".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum Line {
    MarkerComment,
    EmptyComment,
    PreambleComment,
    Empty,
    Content,
}

#[derive(Debug)]
pub enum EssenceError {
    BaseDirExistsError,
    GarbledFunctionNameError(String),
    GarbledMarkerError(u32),
    IoError(io::Error),
    SchemaMissingError(String),
}

impl fmt::Display for EssenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EssenceError::IoError(ref err) => err.fmt(f),
            EssenceError::GarbledMarkerError(line_number) => {
                write!(f, "garbled marker comment (line: {})", line_number)
            }
            EssenceError::SchemaMissingError(ref function) => {
                write!(f, "schema unknown for function '{}'", function)
            }
            EssenceError::GarbledFunctionNameError(ref signature) => {
                write!(f, "function signature garbled ({})", signature)
            }
            EssenceError::BaseDirExistsError => write!(
                f,
                "Base directory already exists, specify '-d' to allow \
                 automcatic deletion."
            ),
        }
    }
}

impl From<io::Error> for EssenceError {
    fn from(err: io::Error) -> EssenceError {
        EssenceError::IoError(err)
    }
}
