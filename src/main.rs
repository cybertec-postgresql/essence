mod lib;

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate structopt;

use lib::types::*;

use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::{Path, PathBuf};

use lib::*;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "essence")]
struct Opt {
    #[structopt(short = "b", long = "base-dir", parse(from_os_str))]
    base_dir: PathBuf,
    /// Automatically re-create the specified base directory.
    #[structopt(short = "r", long = "recreate-base-dir")]
    recreate_base_dir: bool,
    #[structopt(long = "preamble-name", default_value = "preamble.sql")]
    preamble_name: String,
}

fn main() -> Result<(), EssenceError> {
    let opt = Opt::from_args();

    if Path::new(&opt.base_dir).exists() && !&opt.recreate_base_dir {
        return Err(EssenceError::BaseDirExistsError);
    } else {
        remove_dir_all(&opt.base_dir).ok();
        create_dir_all(&opt.base_dir)?;
    }

    let mut mode = Mode::Preamble;

    let mut unwrapped_line: String;

    let mut current_line: Line;
    let mut previous_line = Line::Empty;

    let mut skip_next_empty_line = false;

    let mut line_number: u32 = 0;

    let mut previous_marker = Entity::default();

    let file: File;
    let mut stream: io::BufWriter<File>;

    file = File::create(&opt.base_dir.join(opt.preamble_name))?;
    stream = io::BufWriter::new(file);

    let event_trigger_base_dir = &opt.base_dir.join(EVENT_TRIGGER_FOLDER);
    let extension_base_dir = &opt.base_dir.join(EXTENSION_FOLDER);
    let role_base_dir = &opt.base_dir.join(ROLE_FOLDER);
    let schema_base_dir = &opt.base_dir.join(SCHEMA_FOLDER);

    for line in io::BufReader::new(io::stdin()).lines() {
        line_number += 1;

        unwrapped_line = line?;

        current_line = identify_line(&unwrapped_line);

        if mode == Mode::Preamble {
            if previous_line == Line::Content && current_line == Line::Empty {
                mode = Mode::Body;
            }
            if current_line == Line::Content {
                stream.write((unwrapped_line + "\n").as_bytes())?;
            }
        } else if mode == Mode::Body {
            if current_line == Line::MarkerComment {
                let marker =
                    identify_marker(&unwrapped_line).ok_or_else(|| {
                        EssenceError::GarbledMarkerError(line_number)
                    })?;

                if REQUIRE_SCHEMA[&marker.entity_type] {
                    let schema = &marker.schema.clone().ok_or_else(|| {
                        EssenceError::SchemaMissingError(marker.name.clone())
                    })?;

                    create_dir_all(&schema_base_dir.join(&schema))?;

                    match marker.entity_type {
                        EntityType::Function => {
                            create_dir_all(
                                &schema_base_dir
                                    .join(&schema)
                                    .join(FUNCTION_FOLDER),
                            )?;

                            if previous_marker.entity_type
                                == EntityType::Function
                                && is_polymorph(&marker, &previous_marker)?
                            {
                                stream.write("\n".as_bytes())?;
                            } else {
                                let file = File::create(
                                    &schema_base_dir
                                        .join(&schema)
                                        .join(FUNCTION_FOLDER)
                                        .join(format!(
                                            "{}.{}.sql",
                                            &schema,
                                            extract_function_name(&marker)?
                                        )),
                                )?;
                                redirect_stream(file, &mut stream)?;
                            }
                        }
                        EntityType::Table => create_schema_entity(
                            &schema,
                            &marker.name,
                            &schema_base_dir,
                            TABLE_FOLDER,
                            &mut stream,
                        )?,
                        EntityType::View => create_schema_entity(
                            &schema,
                            &marker.name,
                            &schema_base_dir,
                            VIEW_FOLDER,
                            &mut stream,
                        )?,
                        EntityType::Type => create_schema_entity(
                            &schema,
                            &marker.name,
                            &schema_base_dir,
                            TYPE_FOLDER,
                            &mut stream,
                        )?,
                        _ => {
                            redirect_stream(
                                File::open("/dev/null")?,
                                &mut stream,
                            )?;
                        }
                    }

                    previous_marker = marker;
                } else {
                    match marker.entity_type {
                        EntityType::Schema => {
                            create_dir_all(
                                &schema_base_dir.join(&marker.name),
                            )?;

                            let file = File::create(
                                &schema_base_dir
                                    .join(&marker.name)
                                    .join(format!("{}.sql", &marker.name)),
                            )?;

                            redirect_stream(file, &mut stream)?;
                        }
                        EntityType::Extension => create_entity(
                            &marker.name,
                            &extension_base_dir,
                            &mut stream,
                        )?,
                        EntityType::EventTrigger => create_entity(
                            &marker.name,
                            &event_trigger_base_dir,
                            &mut stream,
                        )?,
                        EntityType::DefaultAcl => create_entity(
                            &marker.owner,
                            &role_base_dir,
                            &mut stream,
                        )?,
                        _ => {
                            redirect_stream(
                                File::open("/dev/null")?,
                                &mut stream,
                            )?;
                        }
                    }
                }

                skip_next_empty_line = true;
            } else if current_line != Line::EmptyComment {
                if !(current_line == Line::Empty && skip_next_empty_line) {
                    stream.write((unwrapped_line + "\n").as_bytes())?;
                } else {
                    skip_next_empty_line = false;
                }
            }
        }

        previous_line = current_line;
    }

    Ok(())
}
