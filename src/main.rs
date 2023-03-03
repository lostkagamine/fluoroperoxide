#[macro_use]
extern crate serde;

pub mod parse;
pub mod directives;
pub mod crates;
pub mod manifest;
pub mod hash;
pub mod shell;
use std::{path::PathBuf, collections::HashMap, process::Stdio};
use clap::{Parser};
use directives::{Directive, Dependency, RustEdition, OptimisationType};
use manifest::{RustToolchainToml, RustToolchainToolchainObj};

use crate::manifest::{CargoToml, CargoTomlPackage, CargoTomlDependencyObj};

// Caching for the... thing
// you know the thing?
// the dependencies, yeah

// you're welcome, crates.io team
#[derive(Serialize, Deserialize)]
pub struct VersionCache {
    pub file_hash: String,
    pub time: std::time::SystemTime,
    pub versions: HashMap<String, String>, // crate -> version mapping
}

#[derive(clap::Parser)]
struct Cli {
    pub file: PathBuf,

    #[clap(short='q',
           long="--quiet",
           help="Suppresses output. Also enables rustc -q.")]
    pub quiet: bool,

    #[clap(long="--force-recache", help="Forces a recache. Only use when necessary.")]
    pub force_recache: bool,
}

#[derive(Debug)]
struct ToolchainInfo {
    pub channel: String,
    pub target: Option<String>,
}

#[derive(Debug)]
struct Project {
    pub dependencies: Vec<Dependency>,
    pub edition: RustEdition,
    pub toolchain: ToolchainInfo,
    pub optimisation: OptimisationType,
}

// Get all initial comments inside a file
fn get_comment_texts(file: &str) -> Vec<String> {
    let mut o = vec![];
    for i in file.split_terminator('\n') {
        let i = i.trim();
        if i.is_empty() {
            continue;
        }

        if i.starts_with("//") {
            let comment_text = i.strip_prefix("//").unwrap();
            let comment_text = comment_text.trim();
            o.push(comment_text.to_owned());
        }
    }
    o
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let log = shell::ShellOutput::new();

    let file = std::fs::read_to_string(&cli.file).unwrap();
    let comments = get_comment_texts(&file);
    // wheeeeeeEEEEEEEE
    let directives = comments.iter()
        .filter_map(|x|
            x.strip_prefix("foof:")
                .and_then(|y| Some(y.trim()))
                .or(None) // it ain't for us
        )
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    let parsed_dirs = directives
        .iter()
        .map(|x| {
            use chumsky::Parser;
            let a = parse::directive().parse_recovery_verbose(x.as_str());
            if let None = a.0 {
                for i in a.1 {
                    println!("{:?}", i);
                }
                panic!("Parsing error");
            }
            a.0.unwrap()
        })
        .collect::<Vec<Directive>>();

    let temp_dir = std::env::temp_dir().join("foof");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let name = cli.file.with_extension("").file_name().unwrap().to_string_lossy().to_string();
    let filepath = cli.file.canonicalize().unwrap().to_string_lossy().to_string();
    let filehash = hash::digest(&filepath);

    let mut prj = Project {
        dependencies: vec![],
        edition: RustEdition::Rust2021,
        toolchain: ToolchainInfo {
            channel: "stable".into(),
            target: None,
        },
        optimisation: OptimisationType::Debug
    };
    for i in &parsed_dirs {
        match i {
            Directive::Dependency(deps) => {
                for j in deps {
                    prj.dependencies.push(j.clone());
                }
            },
            Directive::Toolchain(chan) => {
                prj.toolchain.channel = chan.clone();
            },
            Directive::ToolchainWithTarget(chan, tgt) => {
                prj.toolchain = ToolchainInfo {
                    channel: chan.into(),
                    target: Some(tgt.clone())
                }
            },
            Directive::When(_, _) => todo!(),
            Directive::Edition(ed) => {
                prj.edition = ed.clone();
            },
            Directive::Optimise(x) => {
                prj.optimisation = x.clone();
            },
        }
    }

    // Build us a Cargo.toml
    let mut toml = CargoToml {
        package: CargoTomlPackage {
            name: name.clone(),
            version: "1.0.0".into(),
            edition: prj.edition,
        },
        dependencies: HashMap::new(),
    };

    let temp_dir = temp_dir.join(format!("{}_{}", name, filehash));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let cache_file = temp_dir.join("foof_lock.toml");
    let has_cache = cache_file.exists();

    let src_hash = hash::digest(&file);

    let cache: Option<VersionCache> = if has_cache {
        let a = std::fs::read_to_string(&cache_file).unwrap();
        let t = toml::from_str::<VersionCache>(&a).unwrap();
        Some(t)
    } else {
        None
    };

    let needs_build = {
        !has_cache ||
            cache.as_ref().unwrap().file_hash != src_hash
    };

    const HOUR: u64 = 3600;
    let use_cache = if has_cache && !cli.force_recache {
        cache.as_ref().unwrap().time.elapsed().unwrap() < std::time::Duration::from_secs(HOUR)
    } else {
        false
    };

    if needs_build {
        let mut vers = HashMap::new();
        if !use_cache {
            for i in &prj.dependencies {
                let ver = match &i.version {
                    crates::CrateVersion::Latest => i.get_latest_version().await,
                    crates::CrateVersion::Specific(x) => x.to_string(),
                };
                vers.insert(i.name.clone(), ver);
            }
        } else {
            for i in &prj.dependencies {
                let ver = match &i.version {
                    crates::CrateVersion::Latest =>
                        match cache.as_ref().unwrap().versions.get(&i.name) {
                            Some(x) => x.clone(),
                            None => i.get_latest_version().await
                        }
                    crates::CrateVersion::Specific(x) => x.to_string(),
                };
                vers.insert(i.name.clone(), ver);
            }
        }

        for i in &prj.dependencies {
            let ver = vers.get(&i.name).unwrap().clone();
            if !cli.quiet {
                log.message("Resolving", &format!("{}: {}", i.name, ver));
            }
            toml.dependencies.insert(i.name.clone(), CargoTomlDependencyObj {
                version: ver,
                features: i.features.iter().map(|x| match x {
                    crates::Feature::Enable(x) => x.clone(),
                    crates::Feature::Disable(_) => todo!(),
                }).collect::<Vec<String>>()
            });
        }

        // Let's write the file(s)
        let toml_text = toml::to_string_pretty(&toml).unwrap();

        // Write Cargo.toml
        std::fs::write(temp_dir.join("Cargo.toml"), toml_text).unwrap();

        // Write main.rs
        std::fs::create_dir_all(temp_dir.join("src")).unwrap();
        std::fs::write(temp_dir.join("src").join("main.rs"), &file).unwrap();

        // Emit `rust-toolchain.toml`
        let toolchain_toml = RustToolchainToml {
            toolchain: RustToolchainToolchainObj {
                channel: prj.toolchain.channel
            }
        };
        let toolchain_toml_text = toml::to_string_pretty(&toolchain_toml).unwrap();
        std::fs::write(temp_dir.join("rust-toolchain.toml"), toolchain_toml_text).unwrap();

        // Run `cargo b`
        if !cli.quiet {
            log.message("Building",  &format!("{}", name));
        }

        let mut cargo_b_command = std::process::Command::new("cargo");
        let mut cargo_b_command = cargo_b_command.arg("build");

        if let Some(ref x) = prj.toolchain.target {
            cargo_b_command = cargo_b_command
                .arg("--target")
                .arg(x);
        }

        if let OptimisationType::Release = prj.optimisation {
            cargo_b_command = cargo_b_command
                .arg("--release");
        }

        if cli.quiet {
            cargo_b_command = cargo_b_command
                .arg("-q");
        }
        
        cargo_b_command = cargo_b_command.current_dir(&temp_dir);

        let cargo_b_status = cargo_b_command
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        
        if !cargo_b_status.success() {
            return;
        }
    }

    if !use_cache {
        // Flush the cache
        let mut new_cache = VersionCache {
            file_hash: src_hash,
            time: std::time::SystemTime::now(),
            versions: HashMap::new(),
        };
        for (dep, obj) in &toml.dependencies {
            new_cache.versions.insert(dep.clone(), obj.version.clone());
        }
        let cache_flush = toml::to_string_pretty(&new_cache).unwrap();
        std::fs::write(cache_file, cache_flush).unwrap();
    } else {
        // Flush the cache anyway to update the file hash
        let new_cache = VersionCache {
            file_hash: src_hash,
            time: cache.as_ref().unwrap().time,
            versions: cache.as_ref().unwrap().versions.clone(),
        };
        let cache_flush = toml::to_string_pretty(&new_cache).unwrap();
        std::fs::write(cache_file, cache_flush).unwrap();
    }

    // cargo b is done
    
    if let None = prj.toolchain.target {
        // Hacky way to get the right file extension for an executable
        let ext = std::env::current_exe()
            .unwrap()
            .extension()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or("".into());

        // Let's run it
        let exe_path = temp_dir.join("target")
            .join(prj.optimisation.directory())
            .join(&name)
            .with_extension(ext);

        if !cli.quiet {
            log.message("Running", &format!(
                "`target/{}/{}`",
                prj.optimisation.directory(),
                name
            ));
        }

        let mut run_cmd = std::process::Command::new(exe_path);
        let run_cmd = run_cmd
            .current_dir(std::env::current_dir().unwrap())
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        
        if !run_cmd.success() {
            println!("process exited with error exit code: {}", run_cmd.code().unwrap());
        }
    }

}
