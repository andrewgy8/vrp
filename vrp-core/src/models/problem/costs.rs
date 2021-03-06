#[cfg(test)]
#[path = "../../../tests/unit/models/problem/costs_test.rs"]
mod costs_test;

use crate::construction::heuristics::InsertionContext;
use crate::models::common::*;
use crate::models::problem::{Actor, TargetObjective};
use crate::models::solution::Activity;
use crate::solver::objectives::{TotalRoutes, TotalTransportCost, TotalUnassignedJobs};
use crate::utils::CollectGroupBy;
use hashbrown::HashMap;
use std::cmp::Ordering;
use std::sync::Arc;

/// A hierarchical multi objective for vehicle routing problem.
pub struct ObjectiveCost {
    primary_objectives: Vec<TargetObjective>,
    secondary_objectives: Vec<TargetObjective>,
}

impl ObjectiveCost {
    pub fn new(primary_objectives: Vec<TargetObjective>, secondary_objectives: Vec<TargetObjective>) -> Self {
        Self { primary_objectives, secondary_objectives }
    }
}

impl Objective for ObjectiveCost {
    type Solution = InsertionContext;

    fn total_order(&self, a: &Self::Solution, b: &Self::Solution) -> Ordering {
        match dominance_order(a, b, &self.primary_objectives) {
            Ordering::Equal => dominance_order(a, b, &self.secondary_objectives),
            order @ _ => order,
        }
    }

    fn distance(&self, _a: &Self::Solution, _b: &Self::Solution) -> f64 {
        unreachable!()
    }

    fn fitness(&self, solution: &Self::Solution) -> f64 {
        solution.solution.get_total_cost()
    }
}

impl MultiObjective for ObjectiveCost {
    fn objectives<'a>(&'a self) -> Box<dyn Iterator<Item = &TargetObjective> + 'a> {
        Box::new(self.primary_objectives.iter().chain(self.secondary_objectives.iter()))
    }
}

impl Default for ObjectiveCost {
    fn default() -> Self {
        Self::new(
            vec![Box::new(TotalUnassignedJobs::default()), Box::new(TotalRoutes::default())],
            vec![Box::new(TotalTransportCost::default())],
        )
    }
}

/// Provides the way to get cost information for specific activities done by specific actor.
pub trait ActivityCost {
    /// Returns cost to perform activity.
    fn cost(&self, actor: &Actor, activity: &Activity, arrival: Timestamp) -> Cost {
        let waiting = if activity.place.time.start > arrival { activity.place.time.start - arrival } else { 0.0 };
        let service = self.duration(actor, activity, arrival);

        waiting * (actor.driver.costs.per_waiting_time + actor.vehicle.costs.per_waiting_time)
            + service * (actor.driver.costs.per_service_time + actor.vehicle.costs.per_service_time)
    }

    /// Returns operation time spent to perform activity.
    fn duration(&self, _actor: &Actor, activity: &Activity, _arrival: Timestamp) -> Cost {
        activity.place.duration
    }
}

/// Default activity costs.
pub struct SimpleActivityCost {}

impl Default for SimpleActivityCost {
    fn default() -> Self {
        Self {}
    }
}

impl ActivityCost for SimpleActivityCost {}

/// Provides the way to get routing information for specific locations and actor.
pub trait TransportCost {
    /// Returns transport cost between two locations.
    fn cost(&self, actor: &Actor, from: Location, to: Location, departure: Timestamp) -> Cost {
        let distance = self.distance(actor.vehicle.profile, from, to, departure);
        let duration = self.duration(actor.vehicle.profile, from, to, departure);

        distance * (actor.driver.costs.per_distance + actor.vehicle.costs.per_distance)
            + duration * (actor.driver.costs.per_driving_time + actor.vehicle.costs.per_driving_time)
    }

    /// Returns transport time between two locations.
    fn duration(&self, profile: Profile, from: Location, to: Location, departure: Timestamp) -> Duration;

    /// Returns transport distance between two locations.
    fn distance(&self, profile: Profile, from: Location, to: Location, departure: Timestamp) -> Distance;
}

/// Contains matrix routing data for specific profile and, optionally, time.
pub struct MatrixData {
    /// A routing profile.
    pub profile: Profile,
    /// A timestamp for which routing info is applicable.
    pub timestamp: Option<Timestamp>,
    /// Travel durations.
    pub durations: Vec<Duration>,
    /// Travel distances.
    pub distances: Vec<Distance>,
}

impl MatrixData {
    /// Creates `MatrixData` without timestamp.
    pub fn new(profile: Profile, durations: Vec<Duration>, distances: Vec<Distance>) -> Self {
        Self { profile, timestamp: None, durations, distances }
    }
}

/// Creates time agnostic or time aware routing costs based on matrix data passed.
pub fn create_matrix_transport_cost(costs: Vec<MatrixData>) -> Result<Arc<dyn TransportCost + Send + Sync>, String> {
    if costs.is_empty() {
        return Err("No matrix data found".to_string());
    }

    let size = (costs.first().unwrap().durations.len() as f64).sqrt() as usize;

    if costs.iter().any(|matrix| matrix.distances.len() != matrix.durations.len()) {
        return Err("Distance and duration collections have different length".to_string());
    }

    if costs.iter().any(|matrix| (matrix.distances.len() as f64).sqrt() as usize != size) {
        return Err("Distance lengths don't match".to_string());
    }

    if costs.iter().any(|matrix| (matrix.durations.len() as f64).sqrt() as usize != size) {
        return Err("Duration lengths don't match".to_string());
    }

    Ok(if costs.iter().any(|costs| costs.timestamp.is_some()) {
        Arc::new(TimeAwareMatrixTransportCost::new(costs, size)?)
    } else {
        Arc::new(TimeAgnosticMatrixTransportCost::new(costs, size)?)
    })
}

/// A time agnostic matrix routing costs.
struct TimeAgnosticMatrixTransportCost {
    durations: Vec<Vec<Duration>>,
    distances: Vec<Vec<Distance>>,
    size: usize,
}

impl TimeAgnosticMatrixTransportCost {
    pub fn new(costs: Vec<MatrixData>, size: usize) -> Result<Self, String> {
        let mut costs = costs;
        costs.sort_by(|a, b| a.profile.cmp(&b.profile));

        if costs.iter().any(|costs| costs.timestamp.is_some()) {
            return Err("Time aware routing".to_string());
        }

        if (0..).zip(costs.iter().map(|c| c.profile)).any(|(a, b)| a != b) {
            return Err("Duplicate profiles can be passed only for time aware routing".to_string());
        }

        let (durations, distances) = costs.into_iter().fold((vec![], vec![]), |mut acc, data| {
            acc.0.push(data.durations);
            acc.1.push(data.distances);

            acc
        });

        Ok(Self { durations, distances, size })
    }
}

impl TransportCost for TimeAgnosticMatrixTransportCost {
    fn duration(&self, profile: Profile, from: Location, to: Location, _: Timestamp) -> Duration {
        *self.durations.get(profile as usize).unwrap().get(from * self.size + to).unwrap()
    }

    fn distance(&self, profile: Profile, from: Location, to: Location, _: Timestamp) -> Distance {
        *self.distances.get(profile as usize).unwrap().get(from * self.size + to).unwrap()
    }
}

/// A time aware matrix costs.
struct TimeAwareMatrixTransportCost {
    costs: HashMap<Profile, (Vec<u64>, Vec<MatrixData>)>,
    size: usize,
}

impl TimeAwareMatrixTransportCost {
    /// Creates a new [`TimeAwareMatrixTransportCost`]
    fn new(costs: Vec<MatrixData>, size: usize) -> Result<Self, String> {
        if costs.iter().any(|matrix| matrix.timestamp.is_none()) {
            return Err("Cannot use matrix without timestamp".to_string());
        }

        let costs = costs.into_iter().collect_group_by_key(|matrix| matrix.profile);

        if costs.iter().any(|(_, matrices)| matrices.len() == 1) {
            return Err("Should not use time aware matrix routing with single matrix".to_string());
        }

        let costs = costs
            .into_iter()
            .map(|(profile, mut matrices)| {
                matrices.sort_by(|a, b| (a.timestamp.unwrap() as u64).cmp(&(b.timestamp.unwrap() as u64)));
                let timestamps = matrices.iter().map(|matrix| matrix.timestamp.unwrap() as u64).collect();

                (profile, (timestamps, matrices))
            })
            .collect();

        Ok(Self { costs, size })
    }
}

impl TransportCost for TimeAwareMatrixTransportCost {
    fn duration(&self, profile: Profile, from: Location, to: Location, timestamp: Timestamp) -> Duration {
        let (timestamps, matrices) = self.costs.get(&profile).unwrap();
        let data_idx = from * self.size + to;

        match timestamps.binary_search(&(timestamp as u64)) {
            Ok(matrix_idx) => *matrices.get(matrix_idx).unwrap().durations.get(data_idx).unwrap(),
            Err(matrix_idx) if matrix_idx == 0 => *matrices.first().unwrap().durations.get(data_idx).unwrap(),
            Err(matrix_idx) if matrix_idx == matrices.len() => {
                *matrices.last().unwrap().durations.get(data_idx).unwrap()
            }
            Err(matrix_idx) => {
                let left_matrix = matrices.get(matrix_idx - 1).unwrap();
                let right_matrix = matrices.get(matrix_idx).unwrap();

                let left_value = *matrices.get(matrix_idx - 1).unwrap().durations.get(data_idx).unwrap();
                let right_value = *matrices.get(matrix_idx).unwrap().durations.get(data_idx).unwrap();

                // perform linear interpolation
                let ratio = (timestamp - left_matrix.timestamp.unwrap())
                    / (right_matrix.timestamp.unwrap() - left_matrix.timestamp.unwrap());

                left_value + ratio * (right_value - left_value)
            }
        }
    }

    fn distance(&self, profile: Profile, from: Location, to: Location, timestamp: Timestamp) -> Distance {
        let (timestamps, matrices) = self.costs.get(&profile).unwrap();
        let data_idx = from * self.size + to;

        match timestamps.binary_search(&(timestamp as u64)) {
            Ok(matrix_idx) => *matrices.get(matrix_idx).unwrap().distances.get(data_idx).unwrap(),
            Err(matrix_idx) if matrix_idx == 0 => *matrices.first().unwrap().distances.get(data_idx).unwrap(),
            Err(matrix_idx) if matrix_idx == matrices.len() => {
                *matrices.last().unwrap().distances.get(data_idx).unwrap()
            }
            Err(matrix_idx) => *matrices.get(matrix_idx).unwrap().distances.get(data_idx).unwrap(),
        }
    }
}

fn dominance_order<S>(a: &S, b: &S, objectives: &Vec<Box<dyn Objective<Solution = S> + Send + Sync>>) -> Ordering {
    let mut less_cnt = 0;
    let mut greater_cnt = 0;

    for objective in objectives.iter() {
        match objective.total_order(a, b) {
            Ordering::Less => {
                less_cnt += 1;
            }
            Ordering::Greater => {
                greater_cnt += 1;
            }
            Ordering::Equal => {}
        }
    }

    if less_cnt > 0 && greater_cnt == 0 {
        Ordering::Less
    } else if greater_cnt > 0 && less_cnt == 0 {
        Ordering::Greater
    } else {
        debug_assert!((less_cnt > 0 && greater_cnt > 0) || (less_cnt == 0 && greater_cnt == 0));
        Ordering::Equal
    }
}
