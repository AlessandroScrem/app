use app_wgpu::prelude::App;

use clap::crate_version;
use clap::{AppSettings, Arg};
// use simplelog::{
//     ColorChoice, ConfigBuilder as LogConfigBuilder, LevelFilter, TermLogger, TerminalMode,
// };

use log::LevelFilter;
use std::env;
use log::Level;
use colored::*;

fn format_log(record: &log::Record) -> String {
    let msg = format!("{}", record.args()); // converte fmt::Arguments in String

    match record.level() {
        Level::Error => msg.red().to_string(),
        Level::Warn  => msg.yellow().to_string(),
        Level::Info  => msg.green().to_string(),
        Level::Debug => msg.blue().to_string(),
        Level::Trace => msg.magenta().to_string(),
    }
}

fn init_logger(verbose_count: u64) {
    // Mappa verbose in LevelFilter
    let level = match verbose_count {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    // Costruiamo un logger senza usare env::set_var
    env_logger::Builder::new()
        // Imposta filtri per crate specifici
        .filter_module("app_wgpu", level) // tuo crate
        .filter_module("wgpu", LevelFilter::Info) // wgpu log
        .filter_module("naga", LevelFilter::Warn) // silenzia info/debug di naga
        .format(|buf, record| {
            use std::io::Write;
            let level_str = match record.level() {
                Level::Error => "ERROR".red(),
                Level::Warn => "WARN".yellow(),
                Level::Info => "INFO".green(),
                Level::Debug => "DEBUG".blue(),
                Level::Trace => "TRACE".magenta(),
            };
            // Se livello è Info o inferiore, stampiamo solo [LEVEL] messaggio
            if record.level() <= Level::Info {
                writeln!(buf, "[{}] {}", level_str, format_log(record))
            } else {
                writeln!(
                    buf,
                    "[{}] {} {}:{} -- [{}]",
                    level_str,
                    record.module_path().filter(|m| !m.contains("app_wgpu")).unwrap_or(""),
                    record.file().unwrap_or(""),
                    record.line().unwrap_or(0),
                    format_log(record),
                )
            }
        })
        .init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = clap::App::new("gltf-viewer")
        .version(option_env!("VERSION").unwrap_or(crate_version!()))
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::DeriveDisplayOrder)
        .before_help("Wgpu App viewer\n\nNavigate with the mouse (left/right click + drag, mouse wheel)")
        .arg(Arg::with_name("FILE") // TODO!: re-add URL when fixed...
            .required(false)
            .takes_value(true)
            .help("glTF file name"))
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .multiple(true)
            .help("Enable verbose logging (log level INFO). Can be repeated up to 3 times to increase log level to DEBUG/TRACE)"))
        .arg(Arg::with_name("WIDTH")
            .long("width")
            .short("w")
            .default_value("2400")
            .help("Width in pixels")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .arg(Arg::with_name("HEIGHT")
            .long("height")
            .short("h")
            .default_value("1200")
            .help("Height in pixels")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .get_matches();

    let _source = args.value_of("FILE");

    let width: u32 = args.value_of("WIDTH").unwrap().parse().unwrap();
    let height: u32 = args.value_of("HEIGHT").unwrap().parse().unwrap();

    init_logger(args.occurrences_of("verbose"));

    // let log_level = match args.occurrences_of("verbose") {
    //     0 => LevelFilter::Warn,
    //     1 => LevelFilter::Info,
    //     2 => LevelFilter::Debug,
    //     _ => LevelFilter::Trace,
    // };

    // let _ = TermLogger::init(
    //     log_level,
    //     LogConfigBuilder::new()
    //         .set_time_level(LevelFilter::Off)
    //         .set_target_level(LevelFilter::Warn)
    //         .set_thread_level(LevelFilter::Off)
    //         .build(),
    //     TerminalMode::Stdout,
    //     ColorChoice::Auto,
    // );

    App::new_with_size(width, height).run()
}
