use clap::Parser;
use toml_edit::DocumentMut;

pub mod options;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Cli::parse();
    let trace_level = match args.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(trace_level)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let span = tracing::info_span!("config");
    let _guard = span.enter();
    options::init_conf().expect("Failed to initialize config");
    let mut conf = std::fs::read_to_string("config.toml")
        .expect("Failed to read config file")
        .parse::<DocumentMut>()
        .expect("Failed to parse config file");
    drop(_guard);

    //features init

    let span = tracing::info_span!("database");
    let _guard = span.enter();
    let pool = sqlitedb::innit::get_connection_pool("ramp.sqlite");
    sqlitedb::innit::init_library(
        pool.get().expect("Failed to get connection from pool"),
        &mut conf,
    )
    .expect("Failed to initialize library");
    drop(_guard);
}
