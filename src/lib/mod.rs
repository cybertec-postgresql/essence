pub mod types;

use types::*;

use std::collections::HashMap;

use std::fs::{create_dir_all, File};
use std::io;
use std::mem;
use std::path::Path;

use regex::Regex;

lazy_static! {
    pub static ref MARKER_COMMENT: Regex = Regex::new(
        r"Name:\s([^;]*);\sType:\s([^;]*);\sSchema:\s([^;]*);\sOwner:\s([^;]*)"
    )
    .unwrap();
    pub static ref PREAMBLE_COMMENT: Regex = Regex::new(r"--.+").unwrap();
    pub static ref FUNCTION_SIGNATURE: Regex =
        Regex::new(r"([^(]*)(\([^)]*\))").unwrap();
    pub static ref TYPE_MAPPING: HashMap<&'static str, EntityType> = {
        let mut map = HashMap::new();
        map.insert("ACL", EntityType::Acl);
        map.insert("AGGREGATE", EntityType::Aggregate);
        map.insert("COMMENT", EntityType::Comment);
        map.insert("CHECK CONSTRAINT", EntityType::CheckConstraint);
        map.insert("CONSTRAINT", EntityType::Constraint);
        map.insert("DATABASE", EntityType::Database);
        map.insert("DEFAULT", EntityType::Default);
        map.insert("DEFAULT ACL", EntityType::DefaultAcl);
        map.insert("DOMAIN", EntityType::Domain);
        map.insert("EVENT TRIGGER", EntityType::EventTrigger);
        map.insert("EXTENSION", EntityType::Extension);
        map.insert("FK CONSTRAINT", EntityType::FkConstraint);
        map.insert("FOREIGN TABLE", EntityType::ForeignTable);
        map.insert("FUNCTION", EntityType::Function);
        map.insert("INDEX", EntityType::Index);
        map.insert("MATERIALIZED VIEW", EntityType::MaterializedView);
        map.insert("MATERIALIZED VIEW DATA", EntityType::MaterializedViewData);
        map.insert("POLICY", EntityType::Policy);
        map.insert("ROW SECURITY", EntityType::RowSecurity);
        map.insert("RULE", EntityType::Rule);
        map.insert("SCHEMA", EntityType::Schema);
        map.insert("SEQUENCE", EntityType::Sequence);
        map.insert("SEQUENCE OWNED BY", EntityType::SequenceOwnedBy);
        map.insert("SEQUENCE SET", EntityType::SequenceSet);
        map.insert("TABLE", EntityType::Table);
        map.insert("TABLE DATA", EntityType::TableData);
        map.insert("TRIGGER", EntityType::Trigger);
        map.insert("TYPE", EntityType::Type);
        map.insert("VIEW", EntityType::View);
        map
    };
    pub static ref REQUIRE_SCHEMA: HashMap<EntityType, bool> = {
        let mut map = HashMap::new();
        map.insert(EntityType::Acl, false);
        map.insert(EntityType::Aggregate, true);
        map.insert(EntityType::CheckConstraint, true);
        map.insert(EntityType::Comment, false);
        map.insert(EntityType::Constraint, true);
        map.insert(EntityType::Database, true);
        map.insert(EntityType::Default, true);
        map.insert(EntityType::DefaultAcl, false);
        map.insert(EntityType::Domain, true);
        map.insert(EntityType::EventTrigger, false);
        map.insert(EntityType::Extension, false);
        map.insert(EntityType::FkConstraint, true);
        map.insert(EntityType::ForeignTable, true);
        map.insert(EntityType::Function, true);
        map.insert(EntityType::Index, true);
        map.insert(EntityType::MaterializedView, true);
        map.insert(EntityType::MaterializedViewData, true);
        map.insert(EntityType::Policy, true);
        map.insert(EntityType::RowSecurity, true);
        map.insert(EntityType::Rule, true);
        map.insert(EntityType::Schema, false);
        map.insert(EntityType::Sequence, true);
        map.insert(EntityType::SequenceOwnedBy, true);
        map.insert(EntityType::SequenceSet, true);
        map.insert(EntityType::Table, true);
        map.insert(EntityType::TableData, true);
        map.insert(EntityType::Trigger, true);
        map.insert(EntityType::Type, true);
        map.insert(EntityType::View, true);
        map
    };
}

pub fn identify_line(line: &str) -> Line {
    if MARKER_COMMENT.is_match(line) {
        Line::MarkerComment
    } else if PREAMBLE_COMMENT.is_match(&line) {
        Line::PreambleComment
    } else {
        match line.as_ref() {
            "--" => Line::EmptyComment,
            "" => Line::Empty,
            _ => Line::Content,
        }
    }
}

pub fn identify_marker(marker: &str) -> Option<Entity> {
    let caps = MARKER_COMMENT.captures(&marker)?;

    let name = caps.get(1)?.as_str();
    let entity_type = caps.get(2)?.as_str();
    let schema = caps.get(3)?.as_str();
    let owner = caps.get(4)?.as_str();

    Some(Entity {
        name: name.to_string(),
        entity_type: TYPE_MAPPING.get(entity_type)?.clone(),
        schema: if schema != "-" {
            Some(schema.to_string())
        } else {
            None
        },
        owner: owner.to_string(),
    })
}

pub fn is_polymorph(f1: &Entity, f2: &Entity) -> Result<bool, EssenceError> {
    Ok(if f1.schema != f2.schema {
        false
    } else {
        extract_function_name(&f1)? == extract_function_name(&f2)?
    })
}

pub fn extract_function_name<'a>(
    function: &'a Entity,
) -> Result<&'a str, EssenceError> {
    || -> Option<&str> {
        Some(
            FUNCTION_SIGNATURE
                .captures(&function.name)?
                .get(1)?
                .as_str(),
        )
    }()
    .ok_or_else(|| {
        EssenceError::GarbledFunctionNameError(function.name.clone())
    })
}

pub fn redirect_stream(
    file: File,
    stream: &mut io::BufWriter<File>,
) -> Result<(), io::Error> {
    mem::replace(stream, io::BufWriter::new(file));

    Ok(())
}

pub fn create_schema_entity(
    schema: &str,
    name: &str,
    base_dir: &Path,
    folder_name: &str,
    stream: &mut io::BufWriter<File>,
) -> Result<(), io::Error> {
    create_dir_all(base_dir.join(schema).join(folder_name))?;

    let file = File::create(
        &base_dir
            .join(&schema)
            .join(folder_name)
            .join(format!("{}.{}.sql", &schema, name)),
    )?;
    redirect_stream(file, stream)?;

    Ok(())
}

pub fn create_entity(
    name: &str,
    base_dir: &Path,
    stream: &mut io::BufWriter<File>,
) -> Result<(), io::Error> {
    create_dir_all(&base_dir)?;

    let file = File::create(&base_dir.join(format!("{}.sql", name)))?;

    redirect_stream(file, stream)?;

    Ok(())
}
