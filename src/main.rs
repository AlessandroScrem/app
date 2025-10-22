use app_wgpu::prelude::App;

use clap::crate_version;
use clap::{AppSettings, Arg};
use simplelog::{
    ColorChoice, ConfigBuilder as LogConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

fn main() ->Result<(), Box<dyn std::error::Error>>{
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

    let log_level = match args.occurrences_of("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let _ = TermLogger::init(
        log_level,
        LogConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .build(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    );

    App::new_with_size(width, height).run()
}
