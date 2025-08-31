use clap::Parser;

use crate::config::get_data_dir;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    /// Whether to login or to use existing `env` variables
    #[arg(short, long)]
    pub login: bool,

    /// Whether to store credentials after login
    #[arg(short, long, default_value_t = true)]
    pub store_credentials: bool,

    /// Whether to fetch credentials after login
    #[arg(short, long, default_value_t = false)]
    pub fetch_credentials: bool,
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Data directory: {data_dir_path}"
    )
}
