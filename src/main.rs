use std::error::Error;

mod config;
mod daemon;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let matches = clap::App::new("khonsu (ddns client)")
        .version("0.1")
        .about("A simple DDNS client")
        .arg(clap::Arg::with_name("CONFIG")
            .help("Sets the configuration file")
            .short("c")
            .long("config")
            .required(true)
            .multiple(false)
            .takes_value(true))
        .arg(clap::Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .get_matches();

    // Initialise logging
    let level = match matches.occurrences_of("v") {
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };
    env_logger::Builder::from_default_env()
        .filter(None, level)
        .init();

    // Parse the configuration
    let config_file = matches.value_of("CONFIG").unwrap();
    let config_str = std::fs::read_to_string(config_file)?;
    let config: config::Config = toml::from_str(&config_str).unwrap();

    // Run the daemon
    daemon::run(&config)?;
    Ok(())
}
