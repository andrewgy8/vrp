use crate::construction::states::SolutionContext;
use crate::models::common::IdDimension;
use crate::models::problem::Job;
use crate::models::solution::Route;
use crate::models::Solution;
use std::io::{BufWriter, Error, ErrorKind, Write};
use std::ops::Deref;

pub fn write_solomon_solution<W: Write>(writer: &mut BufWriter<W>, solution: &Solution) -> Result<(), Error> {
    if !solution.unassigned.is_empty() {
        return Err(Error::new(ErrorKind::Other, "Cannot write solomon solution with unassigned jobs."));
    }

    writer.write("Solution\n".as_bytes())?;

    solution.routes.iter().zip(1..).for_each(|(r, i)| {
        let customers = r
            .tour
            .all_activities()
            .filter(|a| a.job.is_some())
            .map(|a| a.job.as_ref().unwrap().as_ref())
            .map(|job| {
                match job {
                    Job::Single(job) => &job.dimens,
                    Job::Multi(job) => &job.dimens,
                }
                .get_id()
                .unwrap()
                .clone()
            })
            .collect::<Vec<String>>()
            .join(" ");
        writer.write(format!("Route {}: {}\n", i, customers).as_bytes()).unwrap();
    });

    Ok(())
}