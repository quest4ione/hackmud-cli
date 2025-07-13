use clap::Args;
use std::{env, fmt::Display, fs, ops::Deref, path::PathBuf, str::FromStr, time::Instant};

use crate::Cli;

/// implement Display and FromStr so i can use it as a default with clap. cause of round-trip
#[derive(Clone)]
struct PathBufArgument(PathBuf);

impl Deref for PathBufArgument {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for PathBufArgument {
    type Err = <PathBuf as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PathBuf::from_str(s).map(Self)
    }
}

impl Display for PathBufArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}
#[cfg(windows)]
fn default_hackmud_path() -> PathBufArgument {
    PathBufArgument(
        env::var_os("APPDATA")
            .map(|p| PathBuf::from(p).join("hackmud"))
            .expect("$APPDATA should be set, Alternatively pass --hackmud-path/set the $HMS_HACKMUD_PATH environment variable"),
    )
}

#[cfg(unix)]
fn default_hackmud_path() -> PathBufArgument {
    PathBufArgument(
        env::var_os("HOME")
            .map(|p| PathBuf::from(p).join(".config/hackmud"))
            .expect("$HOME should be set, Alternatively pass --hackmud-path or set the $HMS_HACKMUD_PATH environment variable"),
    )
}

struct Script {
    name: String,
    path: PathBuf,
}

struct User {
    name: String,
    scripts_path: PathBuf,
}

#[derive(Args)]
pub struct Sync {
    #[arg()]
    scripts: Vec<String>,

    #[cfg(any(windows, unix))]
    #[arg(long, env = "HMS_HACKMUD_PATH", default_value_t=default_hackmud_path())]
    hackmud_path: PathBufArgument,

    #[cfg(not(any(windows, unix)))]
    #[arg(long, env = "HMS_HACKMUD_PATH")]
    hackmud_path: PathBufArgument,
}

impl Sync {
    fn get_scripts(&self) -> Result<Vec<Script>, ()> {
        let mut scripts: Vec<Script> = Vec::new();

        for path_glob in self.scripts.iter() {
            let paths = match glob::glob(path_glob) {
                Ok(paths) => paths,
                Err(e) => {
                    eprintln!("error: failed to parse glob '{path_glob}': {e}");
                    return Err(());
                }
            };

            for path in paths {
                let path = match path {
                    Ok(path) => path,
                    Err(e) => {
                        eprintln!("warn: cant read path: {e}");
                        eprintln!("skipping {}", e.path().to_string_lossy());
                        continue;
                    }
                };

                let ext = path.extension().and_then(|ext| ext.to_str());
                if ext != Some("js") {
                    continue;
                }

                // i cant unwrap here because of ext guarantees because name might contain non-utf8 chars
                let name = match path.file_stem().and_then(|name| name.to_str()) {
                    Some(name) => name,
                    None => continue,
                };

                scripts.push(Script {
                    name: name.to_owned(),
                    path,
                });
            }
        }

        Ok(scripts)
    }

    /// get all users by looking at their name.key files in the hackmud directory
    fn get_users(&self) -> Result<Vec<User>, ()> {
        let mut users: Vec<User> = Vec::new();

        let entries = match fs::read_dir(&*self.hackmud_path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("error: cant read hackmud directory: {e}");
                return Err(());
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("warn: cant read a file in hackmud dir: {e}");
                    continue;
                }
            };

            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();

            let ext = path.extension().and_then(|ext| ext.to_str());
            if ext != Some("key") {
                continue;
            }

            let name = match path.file_stem().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };

            users.push(User {
                scripts_path: self.hackmud_path.join(name).join("scripts"),
                name: name.to_owned(),
            });
        }

        Ok(users)
    }

    pub fn run(&self, _cli: &Cli) -> Result<(), ()> {
        let start = Instant::now();

        let scripts = self.get_scripts()?;

        let users = self.get_users()?;

        for user in users.iter() {
            for script in scripts.iter() {
                let from = &script.path;
                let to = &user.scripts_path.join(format!("{}.js", script.name));

                if let Err(e) = fs::copy(from, to) {
                    eprintln!(
                        "error: couldn't copy script {}.{}, skipping: {e}",
                        user.name, script.name
                    );
                    continue;
                }
            }
        }

        let took = start.elapsed();
        println!(
            "copied {} scripts to {} users in {}ms",
            scripts.len(),
            users.len(),
            took.as_millis()
        );

        Ok(())
    }
}
