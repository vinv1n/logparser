mod parsers;
mod parser_config;
mod decompressors;

use clap::Parser;
use flexi_logger::Logger;
use parsers::log_parser::LogParser;
use log::info;
use std::io::Error;

use parser_config::parser_config::ParserConfig;


// Add log level at somepoint
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CLIArgs {
    #[arg(short, long, help="Path to parser configuration file")]
    config: String,
    #[arg(value_name = "LOGFILES", index = 1, help="Path to log file(s)")]
    logfile_path: String,
    #[arg(short, long, help="Generate a new parser configuration file", require_equals = false)]
    generate_config: bool,
    #[arg(short, long, help="Use verbose logging")]
    verbose: bool
}


fn main() -> Result<(), Error> { 
    let args = CLIArgs::parse();
    let loglevel = match args.verbose {
        true => String::from("debug"),
        false => String::from("info"),
    };

    // this is pure madness no idea why this does not accept loglevels any other way
    // at least I did not get them working. Hopefully there is other way to do this
    Logger::try_with_str(format!("{}, my::critical::module=trace", loglevel)).unwrap()
        .start()
        .unwrap();

    if args.generate_config {
        log::info!("Generating parser config to path {:?}", args.config);
        ParserConfig::generate_template(args.config);
        return Ok(());
    }

    let config = ParserConfig::read_from_file(&args.config);
    info!("Using parser config {}", config);

    let parser = &mut LogParser::new(&args.logfile_path, &config);
    parser.parse().unwrap();

    log::info!("Parsed {} events", parser.event_count());
    for e in parser.iter() {
        // would be nice to have some post functionality in here to post parsed json/yaml entries
        log::info!("Event: {}", e);
    }
    Ok(())
}
