mod parsers;
mod parser_config;
mod decompressors;

use clap::Parser;
use flexi_logger::Logger;
use parsers::log_parser::LogParser;
use log::info;
use std::io::Error;

use parser_config::parser_config::ParserConfig;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CLIArgs {
    #[arg(short, long, help="Path to parser configuration file")]
    config: String,
    #[arg(value_name = "LOGFILES", index = 1, help="Path to log file(s)")]
    logfile_path: String,
    // add this once time to fix arg parser
    #[arg(short, long, help="Generate a new parser configuration file", require_equals = false)]
    generate_config: bool
}


fn main() -> Result<(), Error> {
    Logger::try_with_str("info, my::critical::module=trace").unwrap().start().unwrap();

    let args = CLIArgs::parse();
    if args.generate_config {
        log::info!("Generating parser config to path {:?}", args.config);
        ParserConfig::generate_template(args.config);
        return Ok(());
    }

    let config = ParserConfig::read_from_file(&args.config);
    info!("Using parser config {:?}", config);

    let parser = &mut LogParser::new(&args.logfile_path, &config);
    parser.parse().unwrap();

    log::info!("Parsed {} events", parser.event_count());
    for e in parser.iter() {
        log::info!("Event: {:?}", e.to_json());
    }
    Ok(())
}
