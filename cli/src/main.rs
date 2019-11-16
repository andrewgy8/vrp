mod args;
use self::args::*;

mod formats;
use self::formats::*;

use core::solver::SolverBuilder;
use std::fs::File;
use std::ops::Deref;
use std::process;

use clap::Values;
use std::sync::Arc;

fn main() {
    let formats = get_formats();
    let matches = get_arg_matches(formats.keys().map(|s| s.deref()).collect::<Vec<&str>>());

    // required
    let problem_path = matches.value_of(PROBLEM_ARG_NAME).unwrap();
    let problem_format = matches.value_of(FORMAT_ARG_NAME).unwrap();
    let problem_file = open_file(problem_path, "problem");

    // optional
    let max_generations = matches.value_of(GENERATIONS_ARG_NAME).unwrap().parse::<usize>().unwrap_or_else(|err| {
        eprintln!("Cannot get max generations: '{}'", err.to_string());
        process::exit(1);
    });
    let variation_coefficient = matches
        .value_of(VARIATION_COEFFICIENT_ARG_NAME)
        .unwrap()
        .split(',')
        .map(|line| {
            line.parse::<f64>().unwrap_or_else(|err| {
                eprintln!("Cannot get variation coefficient: '{}'", err.to_string());
                process::exit(1);
            })
        })
        .collect();
    let minimize_routes = matches.value_of(MINIMIZE_ROUTES_ARG_NAME).unwrap().parse::<bool>().unwrap_or_else(|err| {
        eprintln!("Cannot get minimize routes: '{}'", err.to_string());
        process::exit(1);
    });
    let init_solution = matches.value_of(INIT_SOLUTION_ARG_NAME).map(|path| open_file(path, "init solution"));
    let matrix_files = matches
        .values_of(MATRIX_ARG_NAME)
        .map(|paths: Values| paths.map(|path| open_file(path, "routing matrix")).collect());

    match formats.get(problem_format) {
        Some((problem_reader, init_reader, solution_writer)) => {
            match problem_reader.0(problem_file, matrix_files) {
                Ok(problem) => {
                    let problem = Arc::new(problem);
                    let solution = init_solution.and_then(|file| init_reader.0(file, problem.clone()));
                    let solution = SolverBuilder::default()
                        .with_init_solution(solution.map(|s| (problem.clone(), Arc::new(s))))
                        .with_minimize_routes(minimize_routes)
                        .with_max_generations(max_generations)
                        .with_variation_coefficient(variation_coefficient)
                        .build()
                        .solve(problem.clone());
                    match solution {
                        Some(solution) => solution_writer.0(&problem, solution.0).unwrap(),
                        None => println!("Cannot find any solution"),
                    };
                }
                Err(error) => {
                    eprintln!("Cannot read {} problem from '{}': '{}'", problem_format, problem_path, error);
                    process::exit(1);
                }
            };
        }
        None => {
            eprintln!("Unknown format: '{}'", problem_format);
            process::exit(1);
        }
    }
}

fn open_file(path: &str, description: &str) -> File {
    File::open(path).unwrap_or_else(|err| {
        eprintln!("Cannot open {} file '{}': '{}'", description, path, err.to_string());
        process::exit(1);
    })
}
