use soup::prelude::*;
use structopt::StructOpt;

use wikinet::*;

#[derive(Debug, StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
/// Follow the first link on Wikipedia till you reach philosophy
struct Opts {
    /// Starting point
    #[structopt(default_value = "Special:Random")]
    start: String,

    /// Verbosity
    #[structopt(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    setup();

    let opts = Opts::from_args();
    tracing::debug!(?opts, "starting with");

    let mut href = ensure_wiki_at_start(trim_wiki_website(&opts.start));

    println!("{:3}: {:32}", 0, href);

    for i in 1..100 {
        if href == "/wiki/Philosophy" {
            println!("END");
            break;
        }
        let text = get_wiki(&href).await?;
        if let Some(link) = get_first_link(&text) {
            href = link.get("href").expect("cannot get href");
            tracing::info!(?href);
            println!("{:3}: {:32} ({})", i, href, link.text().trim());
        } else {
            break;
        }
    }
    Ok(())
}

fn setup() {
    use tracing_subscriber::EnvFilter;
    // if std::env::var("RUST_LOG").is_err() {
    //     std::env::set_var("RUST_LOG", "warn,wikinet=debug")
    // }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_thread_ids(true)
        .with_thread_names(true)
        .without_time()
        .init();

    let _span = tracing::debug_span!("setup").entered();
    tracing::trace!("tracing_subscriber installed");

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install().expect("cannot initialize color_eyre");
    tracing::trace!("color_eyre installed");

    tracing::debug!("setup performed");
}
