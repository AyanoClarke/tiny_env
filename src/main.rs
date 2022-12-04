use clap::Parser;
use env_logger;
use log;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use toml;

#[derive(Parser)]
#[command(author, version, about, arg_required_else_help = true)]
struct Cli {
    /// toml format software config file.
		/// toml file contains multiple endpoints, with environment as keyword.
    #[arg(short = 'c', long, value_name = "CONFIG")]
    software_config: Option<PathBuf>,

    /// endpoints
    endpoint: Vec<String>,
}

fn read_config(config_path: &PathBuf) -> std::io::Result<toml::Value> {
    let content = std::fs::read_to_string(config_path)?;
    Ok(toml::from_str(&content)?)
}

/// get all endpoints from requirements.
fn get_all_endpoints(
    config: &toml::Value,
    current_endpoing_set: HashSet<String>,
) -> HashSet<String> {
    let mut new_added: HashSet<String> = HashSet::new();
    let mut results: HashSet<String> = HashSet::new();
    for endpoint in &current_endpoing_set {
        results.insert(endpoint.to_string());
        if let Some(ed_config) = config.get(&endpoint) {
            if let Some(requires) = ed_config.get("require") {
                let requires: &Vec<toml::Value> = requires.as_array().unwrap();
                for require in requires {
                    let require_endpoint = require.as_str().unwrap();
                    if !current_endpoing_set.contains(require_endpoint) {
                        new_added.insert(require_endpoint.to_owned());
                    }
                    results.insert(require_endpoint.to_string());
                }
            }
        }
    }
    if new_added.len() > 0 {
        return get_all_endpoints(config, results);
    } else {
        return current_endpoing_set;
    }
}

fn main() {
    let default_conda_path: std::string::String = String::from("$HOME/conda");

    let cli = Cli::parse();

    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    let software_config = cli.software_config.as_ref().unwrap();

    log::info!(
        "Software config is {:?}, selected endpoints are {:?}",
        software_config,
        cli.endpoint
    );

    let config = read_config(software_config).unwrap();
    log::debug!("config contents is \"{:?}\".", config);

    let base_conda_path = PathBuf::from(
        config
            .get("conda")
            .unwrap_or(&toml::Value::String(default_conda_path))
            .as_str()
            .unwrap(),
    );
    log::info!("base conda environment is {:?}", base_conda_path);

    let endpoints = get_all_endpoints(&config, cli.endpoint.into_iter().collect());
    log::info!("All requirements is {:?}", endpoints);

    let mut outf = File::create("env.profile").unwrap();
    outf.write_all(
        format!(
            "# software config is {:?} activate modules {:?}\n",
            &cli.software_config.unwrap(),
            endpoints
        )
        .as_bytes(),
    )
    .unwrap();

    let mut conda_env: Option<PathBuf> = None;
    let mut envs: HashMap<String, String> = HashMap::new();
    for endpoint in endpoints {
        if let Some(cfg_map) = config.get(endpoint) {
            let cfg_table = cfg_map.as_table().unwrap();
            for key in cfg_table.keys() {
                match key.to_owned().as_ref() {
                    "require" => {}
                    // conda environments
                    "conda" => match conda_env {
                        Some(_) => {
                            log::warn!("conda env has more than 1 path.")
                        }
                        None => {
                            conda_env =
                                Some(PathBuf::from(cfg_map.get(key).unwrap().as_str().unwrap()));
                        }
                    },
                    _ => {
                        log::debug!("{:?}: {:?}", key, cfg_table.get(key).unwrap());
                        if envs.contains_key(key) {
                            envs.entry(key.to_string()).and_modify(|v| {
                                *v = format!(
                                    "{}:{}",
                                    *v,
                                    cfg_table.get(key).unwrap().as_str().unwrap()
                                )
                            });
                        } else {
                            envs.insert(
                                key.to_string(),
                                cfg_table.get(key).unwrap().as_str().unwrap().to_string(),
                            );
                        }
                    }
                }
            }
        }
    }
    log::debug!("results is {:?}", envs);

    // output conda environments
    match conda_env {
        Some(cenv) => {
            let activate = base_conda_path.join("bin").join("activate");
            outf.write_all(
                format!("source {} {}\n", activate.display(), cenv.display()).as_bytes(),
            )
            .unwrap();
            log::debug!("source {} {}", activate.display(), cenv.display());
        }
        None => {}
    }
    // output other environments
    for key in envs.keys() {
        outf.write_all(format!("export {}={}:${}\n", key, envs.get(key).unwrap(), key).as_bytes())
            .unwrap();
        log::debug!("export {}={}:${}", key, envs.get(key).unwrap(), key);
    }
}
