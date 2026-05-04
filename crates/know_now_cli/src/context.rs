use std::path::PathBuf;

use crate::output::OutputFormat;

pub struct CommandContext {
    pub format: OutputFormat,
    pub verbose: bool,
    pub debug: bool,
    pub no_color: bool,
    pub project_root: PathBuf,
    pub config_path: Option<PathBuf>,
}
