use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// DDNS using Cloudflare API
pub(crate) struct Args {
    /// the path to config file
    #[argh(option, short = 'c')]
    pub config: String,

    /// whether to run in daemon mode
    #[argh(switch, short = 'd')]
    pub daemon: bool,
}