use super::*;
use std::io::BufReader;
use std::process;
use vrp_cli::extensions::generate::generate_problem;
use vrp_pragmatic::format::problem::serialize_problem;

pub const FORMAT_ARG_NAME: &str = "FORMAT";
pub const INPUT_ARG_NAME: &str = "input-files";
pub const OUT_RESULT_ARG_NAME: &str = "out-result";
pub const JOBS_SIZE_ARG_NAME: &str = "jobs-size";

pub fn get_generate_app<'a, 'b>() -> App<'a, 'b> {
    App::new("generate")
        .about("Provides the way to generate meaningful problems for testing")
        .arg(
            Arg::with_name(FORMAT_ARG_NAME)
                .help("Specifies input type")
                .required(true)
                .possible_values(&["pragmatic"])
                .index(1),
        )
        .arg(
            Arg::with_name(INPUT_ARG_NAME)
                .help("Sets input files which contains a VRP definition prototype")
                .short("i")
                .long(INPUT_ARG_NAME)
                .required(true)
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name(OUT_RESULT_ARG_NAME)
                .help("Specifies path to file for result output")
                .short("o")
                .long(OUT_RESULT_ARG_NAME)
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(JOBS_SIZE_ARG_NAME)
                .help("Amount of jobs in the plan of generated problem")
                .short("j")
                .long(JOBS_SIZE_ARG_NAME)
                .required(true)
                .takes_value(true),
        )
}

pub fn run_generate(matches: &ArgMatches) {
    let input_format = matches.value_of(FORMAT_ARG_NAME).unwrap();
    let input_files = matches
        .values_of(INPUT_ARG_NAME)
        .map(|paths: Values| paths.map(|path| BufReader::new(open_file(path, "input"))).collect::<Vec<_>>());
    let jobs_size = matches
        .value_of(JOBS_SIZE_ARG_NAME)
        .map(|arg| {
            arg.parse::<usize>().unwrap_or_else(|err| {
                eprintln!("cannot get jobs size: '{}'", err.to_string());
                process::exit(1);
            })
        })
        .unwrap();

    match generate_problem(input_format, input_files, jobs_size) {
        Ok(problem) => {
            let out_result = matches.value_of(OUT_RESULT_ARG_NAME).map(|path| create_file(path, "out result"));
            let out_buffer = create_write_buffer(out_result);
            if let Err(err) = serialize_problem(out_buffer, &problem) {
                eprintln!("Cannot serialize result problem: '{}'", err);
                process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("Cannot generate problem: '{}'", err);
            process::exit(1);
        }
    }
}
