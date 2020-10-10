use clap::{App, Arg};
use reesolve::Input;
use reesolve::Resolver;
use reesolve::Result;
use std::path::{Path, PathBuf};

fn create_clap_app(version: &str) -> clap::App {
    App::new("reesolve")
        .version(version)
        .about("A DNS resolver written in Rust")
        .usage("cat hosts.txt | ree")
        .arg(
            Arg::with_name("input-file")
                .help("ree -i <hosts.txt>")
                .short("i")
                .long("input-file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("resolvers")
                .help("ree -r <resolvers.txt>\nThe default list of resolvers used is Google & CloudFlare.")
                .short("r")
                .long("resolvers")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("concurrency")
                .help("ree -i hosts.txt -c 200")
                .short("c")
                .long("concurrency")
                .default_value("320")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbosity")
                .help("ree -i hosts.txt -v info")
                .short("v")
                .long("verbosity")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .help("ree -i hosts.txt -t 10")
                .short("t")
                .long("timeout")
                .default_value("5")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .help(
                    "ree -i hosts.txt -o /some/path/file\nWill automatically add the .json extension to the file.",
                )
                .short("o")
                .long("output")
                .default_value("records")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output-format")
                .help("ree -f csv")
                .short("-f")
                .long("output-format")
                .default_value("json")
                .takes_value(true),
        )
}

fn make_path(path: &str, format: &str) -> PathBuf {
    let path = Path::new(path);
    let file = path.file_name().unwrap().to_str().unwrap();
    path.with_file_name(format!("{}.{}", file, format))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = create_clap_app("0.0.2");
    let matches = args.get_matches();
    let concurrency: usize = matches.value_of("concurrency").unwrap().parse()?;
    let timeout: u64 = matches.value_of("timeout").unwrap().parse()?;
    let input_file = matches.value_of("input-file");
    let output_format = matches.value_of("output-format").unwrap();
    let output_path = make_path(matches.value_of("output").unwrap(), output_format);
    let targets = Input::new(input_file).hosts();

    if matches.is_present("verbosity") {
        let builder = tracing_subscriber::fmt()
            .with_env_filter(matches.value_of("verbosity").unwrap())
            .with_filter_reloading();
        let _handle = builder.reload_handle();
        builder.try_init()?;
    }

    // if the user specified a list of resolvers, use them.
    let ree = Resolver::default();
    if matches.is_present("resolvers") {
        let resolvers = matches.value_of("resolvers").unwrap();
        ree.load_resolvers(resolvers)
            .timeout(timeout)
            .output(output_format, output_path)
            .resolve(targets, concurrency)
            .await?;
    } else {
        ree.timeout(timeout)
            .output(output_format, output_path)
            .resolve(targets, concurrency)
            .await?;
    }

    Ok(())
}
