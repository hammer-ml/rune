#![allow(deprecated)]

use log;
use pest::Parser;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use cargo::ops::VersionControl;
use std::borrow::ToOwned;

use std::io::Write;

mod runegen;

mod instructions {
    pub mod capability;
    pub mod from;
    pub mod model;
    pub mod out;
    pub mod proc_block;
    pub mod run;
}

#[derive(Parser)]
#[grammar = "runefile.pest"]
pub struct RunefileParser;

#[derive(Debug)]
pub enum Instruction {
    FromInstruction(instructions::from::FromInstruction),
    Model(instructions::model::ModelInstruction),
    Capability(instructions::capability::CapabilityInstruction),
    Run(instructions::run::RunInstruction),
    ProcBlock(instructions::proc_block::ProcBlockInstruction),
    Out(instructions::out::OutInstruction),
    //Misc(MiscInstruction)
}

pub fn generate(contents: String) -> PathBuf {
    let instructions = parse(contents);
    //Iterate through all instructions

    let (dependencies, proc_options, capability_manifest, models_manifest, outtype_manifest) =
        process_instructions(instructions);

    //Get homedir
    let mut homedir = match env::home_dir() {
        None => panic!(""),
        Some(p) => p,
    };

    //add directories to homedir if not exist
    homedir.push(".rune");
    homedir.push("runes");

    let runedir = match homedir.to_str().map(ToOwned::to_owned) {
        Some(p) => p,
        None => panic!(""),
    };

    match fs::create_dir_all(&runedir) {
        Ok(()) => (),
        _ => {}
    }

    //Cargo

    //set up config
    let config = match cargo::util::Config::default() {
        Ok(con) => con,
        Err(err) => {
            log::error!("Config doesn't work {}", err);
            std::process::exit(1);
        }
    };

    let mut runedir = PathBuf::from(runedir);
    let rune_uuid = uuid::Uuid::new_v4();

    runedir.push(format!("{}", rune_uuid));

    let runedir_out = match runedir.to_str().map(ToOwned::to_owned) {
        Some(p) => p,
        None => panic!("Cannot own rune_dir string"),
    };
    //make cargo dir
    let opts = match cargo::ops::NewOptions::new(
        Some(VersionControl::Git),
        false,
        true,
        runedir.clone(),
        Some(String::from("rune")),
        None,
        None,
    ) {
        Ok(opts) => opts,
        Err(err) => {
            log::error!("Failed to make rune cargo {}", err);
            std::process::exit(1);
        }
    };

    match cargo::ops::init(&opts, &config) {
        Ok(_) => log::debug!("Cargo project created"),
        Err(err) => {
            log::error!("Cargo init failed {:?}", err);
            std::process::exit(1);
        }
    }

    //Cargo.toml
    let mut cargo_toml = String::from("");

    // We need to use toml editor here

    cargo_toml = [
        format!("\n[lib]\ncrate-type = [\"cdylib\"]\n"),
        String::from(cargo_toml),
    ]
    .concat();

    for key in dependencies.keys() {
        cargo_toml = [
            format!("{} = {}\n", key, dependencies[key]),
            String::from(cargo_toml),
        ]
        .concat();
    }
    write_to_file(
        format!(
            "{}/Cargo.toml",
            runedir.clone().as_path().display().to_string()
        ),
        cargo_toml,
    );

    // temp generate sine function
    write_to_file(
        format!(
            "{}/src/sine_model.rs",
            runedir.clone().as_path().display().to_string()
        ),
        runegen::sine_model(),
    );

    //generate lib.rs
    let lib_code: String = [
        runegen::generate_code(runegen::CodeChunk::Attributes, None),
        runegen::generate_code(runegen::CodeChunk::Header, None),
        //how about this ugly temp fix
        String::from("mod sine_model;\n\n"),
        runegen::generate_code(runegen::CodeChunk::TfmModelInvoke, None),
        runegen::generate_code(runegen::CodeChunk::Malloc, None),
        runegen::generate_code(runegen::CodeChunk::RuneBufferPtr, None),
        runegen::generate_code(runegen::CodeChunk::ProviderResponsePtr, None),
        runegen::generate_code(runegen::CodeChunk::ManifestFn, None),
        runegen::generate_manifest_function(capability_manifest, models_manifest, outtype_manifest),
        runegen::generate_code(runegen::CodeChunk::Call, Some(proc_options)),
    ]
    .concat();

    write_to_file(
        format!(
            "{}/src/lib.rs",
            runedir.clone().as_path().display().to_string()
        ),
        lib_code,
    );
    log::info!("Succesfully generated rune container in {:?}", runedir);
    return PathBuf::from(runedir_out);
}

fn write_to_file(file: String, content: String) {
    let mut file_ref = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(format!("{}", file))
        .unwrap();
    if let Err(e) = writeln!(file_ref, "{}\n", content) {
        log::error!("Couldn't write to file: {}", e);
        std::process::exit(1);
    }
}

pub fn parse(contents: String) -> HashMap<String, Instruction> {
    let successful_parse = RunefileParser::parse(Rule::runefile, &contents)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    let mut results: HashMap<String, Instruction> = HashMap::new();
    let mut counter = 0;
    for record in successful_parse.into_inner() {
        match record.as_rule() {
            Rule::step => {
                for step in record.into_inner() {

                    match step.as_rule() {
                        Rule::from => {
                            
                            results.insert(
                                counter.to_string(),
                                Instruction::FromInstruction(
                                    instructions::from::FromInstruction::from_record(step),
                                ),
                            );
                        }
                        Rule::model => {
                            results.insert(
                                counter.to_string(),
                                Instruction::Model(
                                    instructions::model::ModelInstruction::from_record(step),
                                ),
                            );
                        }
                        Rule::capability => {
                            results.insert(
                                counter.to_string(),
                                Instruction::Capability(
                                    instructions::capability::CapabilityInstruction::from_record(
                                        step,
                                    ),
                                ),
                            );
                        }
                        Rule::run => {
                            results.insert(
                                counter.to_string(),
                                Instruction::Run(instructions::run::RunInstruction::from_record(
                                    step,
                                )),
                            );
                        }
                        Rule::proc_line => {
                            results.insert(
                                counter.to_string(),
                                Instruction::ProcBlock(
                                    instructions::proc_block::ProcBlockInstruction::from_record(
                                        step,
                                    ),
                                ),
                            );
                        }
                        Rule::out => {
                            results.insert(
                                counter.to_string(),
                                Instruction::Out(instructions::out::OutInstruction::from_record(
                                    step,
                                )),
                            );
                        }
                        _ => {
                            log::error!("Step doesn't follow expected grammar: {}", step.as_str());
                            //TODO: Let's add the original text
                            std::process::exit(1);
                        }
                    }
                    counter += 1;
                }
            }
            _ => (),
        }
    }
    return results;
    // Make folder
}

pub fn process_instructions(
    instructions: HashMap<String, Instruction>,
) -> (
    HashMap<String, String>,
    HashMap<String, String>,
    HashMap<String, String>,
    HashMap<String, String>,
    String,
) {
    let mut dependencies: HashMap<String, String> = HashMap::new();
    let mut proc_options: HashMap<String, String> = HashMap::new();
    let mut capability_manifest: HashMap<String, String> = HashMap::new();
    let mut models_manifest: HashMap<String, String> = HashMap::new();
    let mut outtype_manifest: String = String::from("serial");

    for elem in instructions.values() {
        match elem {
            Instruction::FromInstruction(value) => {
                let from_instruction: &instructions::from::FromInstruction = value;
                log::info!("{:?}", from_instruction);
            }
            Instruction::Model(value) => {
                let model_instruction: &instructions::model::ModelInstruction = value;
                log::info!("{:?}", model_instruction);
                dependencies.extend(model_instruction.dependencies.clone());
                models_manifest.insert(
                    model_instruction.model_name.clone(),
                    model_instruction.code.clone(),
                );
            }
            Instruction::Capability(value) => {
                let capability_instruction: &instructions::capability::CapabilityInstruction =
                    value;
                capability_manifest.insert(
                    capability_instruction.capability_name.clone(),
                    capability_instruction.code.clone(),
                );
                log::info!("{:?}", capability_instruction);
            }
            Instruction::Run(value) => {
                let run_instruction: &instructions::run::RunInstruction = value;
                log::info!("{:?}", run_instruction);
            }
            Instruction::ProcBlock(value) => {
                let proc_instruction: &instructions::proc_block::ProcBlockInstruction = value;
                log::info!("{:?}", proc_instruction);
                dependencies.extend(proc_instruction.dependencies.clone());
                proc_options.extend(proc_instruction.params.clone());
            }
            Instruction::Out(value) => {
                let out_instruction: &instructions::out::OutInstruction = value;
                log::info!("{:?}", out_instruction);
                outtype_manifest = out_instruction.out_type.clone();
            }
        }
    }

    (
        dependencies,
        proc_options,
        capability_manifest,
        models_manifest,
        outtype_manifest,
    )
}

pub(crate) type Pair<'a> = pest::iterators::Pair<'a, Rule>;
