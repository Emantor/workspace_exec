extern crate i3ipc;
use i3ipc::I3Connection;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::env::set_current_dir;

use std::iter::Iterator;

extern crate clap;
#[macro_use] extern crate failure;
use clap::{App, Arg};

extern crate yaml_rust;
use yaml_rust::{YamlLoader, Yaml};

extern crate shellexpand;
use shellexpand::tilde;

use failure::Error;
use failure::err_msg;

#[derive(Debug, Fail)]
enum WorkspaceExecError {
    #[fail(display = "No arguments supplied: {}", name)]
    NoArgumentError {
        name: String,
    }
}

fn main() {
    let matches = App::new("workspace_exec").version("0.1")
                              .about("executes program in path using a yaml mapping from i3/sway workspace name")
                              .author("Rouven Czerwinski <r.czerwinski@pengutronix.de")
                              .arg(Arg::with_name("config")
                                   .short("c")
                                   .long("config")
                                   .value_name("FILE")
                                   .help("Sets the mapping configuration file"))
                              .arg(Arg::with_name("args")
                                   .multiple(true)
                                   .last(true)
                                   .value_name("Args")
                                   .help("Program Arguments"))
                              .get_matches();
    /* Create a lambda which handles the Errors or Results from the function.
     * Error Handling is than done in the if let Err down below.
     * This enables us to use ? inside main. */
    let err = || -> Result<(), Error> {
        let mut connection = I3Connection::connect()?;

        let config = matches.value_of("config").unwrap_or("~/.config/sway/workspace_exec/mapping.yaml");
        let config = tilde(config).to_string();
        let config_path = Path::new(&config);
        change_dir_from_mapping(&config_path, &mut connection)?;
        let args = matches.values_of("args")
                          .ok_or(WorkspaceExecError::NoArgumentError{ name: "No arguments found in input".to_string() })?;
        let mut args: std::vec::Vec<String> = args.collect::<Vec<_>>().into_iter().map(|s| s.to_owned()).collect();
        let binary = args.remove(0);
        std::process::Command::new(binary).args(&args).spawn()?;
        Ok(())
    }();

    if let Err(e) = err {
        println!("{}", err_msg(e));
        std::process::exit(1);
    }


}

fn open_parse_config(path: &Path) -> Result<std::vec::Vec<Yaml>, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let yaml = YamlLoader::load_from_str(&contents)?;
    Ok(yaml)
}

fn get_active_workspace(conn: &mut I3Connection) -> Result<String, Error> {
    let workspaces = conn.get_workspaces()?;
    Ok(workspaces.workspaces.iter().filter(|x| x.focused).map(|x| x.name.to_string()).collect())
}

fn change_dir_from_mapping(config: &Path, mut conn: &mut I3Connection) -> Result<bool, Error> {
    let workspace = get_active_workspace(&mut conn)?;
    let mapping = open_parse_config(&config);
    if let Err(e) = mapping {
        println!("Config Error: {}", err_msg(e));
        return Ok(false)
    }
    let mapping = mapping.unwrap();
    let dir = match mapping[0]["mapping"][&workspace[..]].clone().into_string() {
        Some(s) => tilde(&s).to_string(),
        None => tilde("~").to_string(),
    };
    match set_current_dir(dir) {
        Ok(_) => Ok(true),
        Err(e) => {
            println!("Could not set dir: {}", err_msg(e));
            std::process::exit(1);
        }
    }
}
