use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// DDNS using Cloudflare API
pub(crate) struct Args {
    /// path to the config file
    #[argh(option, short = 'c')]
    pub config: String,

    /// path to the file containing auth token
    #[argh(option, short = 't')]
    pub token: String,

    /// whether to run in daemon mode
    #[argh(switch, short = 'd')]
    pub daemon: bool,
}