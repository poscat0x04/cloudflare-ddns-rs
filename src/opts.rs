use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(short, long)]
    pub config: String,
    #[clap(short, long)]
    pub oneshot: bool,
}
