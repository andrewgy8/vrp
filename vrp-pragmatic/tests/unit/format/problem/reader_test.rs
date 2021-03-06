use super::create_approx_matrices;
use crate::extensions::MultiDimensionalCapacity;
use crate::format::problem::*;
use crate::helpers::*;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::Arc;
use vrp_core::construction::constraints::{Demand, DemandDimension};
use vrp_core::models::common::{Dimensions, IdDimension, TimeSpan, TimeWindow};
use vrp_core::models::problem::{Jobs, Multi, Place, Single};

fn get_job(index: usize, jobs: &Jobs) -> vrp_core::models::problem::Job {
    jobs.all().collect::<Vec<_>>().get(index).unwrap().clone()
}

fn get_single_job(index: usize, jobs: &Jobs) -> Arc<Single> {
    get_job(index, jobs).to_single().clone()
}

fn get_multi_job(index: usize, jobs: &Jobs) -> Arc<Multi> {
    get_job(index, jobs).to_multi().clone()
}

fn get_single_place(single: &Single) -> &Place {
    single.places.first().unwrap()
}

fn assert_time_window(tw: &TimeWindow, expected: &(f64, f64)) {
    assert_eq!(tw.start, expected.0);
    assert_eq!(tw.end, expected.1);
}

fn assert_time_spans(tws: &Vec<TimeSpan>, expected: Vec<(f64, f64)>) {
    assert_eq!(tws.len(), expected.len());
    (0..tws.len()).for_each(|index| {
        assert_time_window(&tws.get(index).and_then(|tw| tw.as_time_window()).unwrap(), expected.get(index).unwrap());
    });
}

fn assert_demand(demand: &Demand<MultiDimensionalCapacity>, expected: &Demand<MultiDimensionalCapacity>) {
    assert_eq!(demand.pickup.0.as_vec(), expected.pickup.0.as_vec());
    assert_eq!(demand.pickup.1.as_vec(), expected.pickup.1.as_vec());
    assert_eq!(demand.delivery.0.as_vec(), expected.delivery.0.as_vec());
    assert_eq!(demand.delivery.1.as_vec(), expected.delivery.1.as_vec());
}

fn assert_skills(dimens: &Dimensions, expected: Option<Vec<String>>) {
    let skills = dimens.get("skills").and_then(|any| any.downcast_ref::<HashSet<String>>());
    if let Some(expected) = expected {
        let expected = HashSet::from_iter(expected.iter().cloned());
        assert_eq!(skills.unwrap().clone(), expected);
    } else {
        assert!(skills.is_none());
    }
}

#[test]
fn can_read_complex_problem() {
    let problem = Problem {
        plan: Plan {
            jobs: vec![
                Job {
                    id: "delivery_job".to_string(),
                    pickups: None,
                    deliveries: Some(vec![JobTask {
                        places: vec![JobPlace {
                            times: Some(vec![
                                vec!["1970-01-01T00:00:00Z".to_string(), "1970-01-01T00:01:40Z".to_string()],
                                vec!["1970-01-01T00:01:50Z".to_string(), "1970-01-01T00:02:00Z".to_string()],
                            ]),
                            location: vec![52.48325, 13.4436].to_loc(),
                            duration: 100.0,
                        }],
                        demand: Some(vec![0, 1]),
                        tag: Some("my_delivery".to_string()),
                    }]),
                    replacements: None,
                    services: None,
                    priority: None,
                    skills: Some(vec!["unique".to_string()]),
                },
                Job {
                    id: "pickup_delivery_job".to_string(),
                    pickups: Some(vec![JobTask {
                        places: vec![JobPlace {
                            times: Some(vec![vec![
                                "1970-01-01T00:00:10Z".to_string(),
                                "1970-01-01T00:00:30Z".to_string(),
                            ]]),
                            location: vec![52.48300, 13.4420].to_loc(),
                            duration: 110.0,
                        }],
                        demand: Some(vec![2]),
                        tag: None,
                    }]),
                    deliveries: Some(vec![JobTask {
                        places: vec![JobPlace {
                            times: Some(vec![vec![
                                "1970-01-01T00:00:50Z".to_string(),
                                "1970-01-01T00:01:00Z".to_string(),
                            ]]),
                            location: vec![52.48325, 13.4436].to_loc(),
                            duration: 120.0,
                        }],
                        demand: Some(vec![2]),
                        tag: None,
                    }]),
                    replacements: None,
                    services: None,
                    priority: None,
                    skills: None,
                },
                Job {
                    id: "pickup_job".to_string(),

                    pickups: Some(vec![JobTask {
                        places: vec![JobPlace {
                            times: Some(vec![vec![
                                "1970-01-01T00:00:10Z".to_string(),
                                "1970-01-01T00:01:10Z".to_string(),
                            ]]),
                            location: vec![52.48321, 13.4438].to_loc(),
                            duration: 90.0,
                        }],
                        demand: Some(vec![3]),
                        tag: None,
                    }]),
                    deliveries: None,
                    replacements: None,
                    services: None,
                    priority: None,
                    skills: Some(vec!["unique2".to_string()]),
                },
            ],
            relations: Option::None,
        },
        fleet: Fleet {
            vehicles: vec![VehicleType {
                type_id: "my_vehicle".to_string(),
                vehicle_ids: vec!["my_vehicle_1".to_string(), "my_vehicle_2".to_string()],
                profile: "car".to_string(),
                costs: VehicleCosts { fixed: Some(100.), distance: 1., time: 2. },
                shifts: vec![VehicleShift {
                    start: VehiclePlace {
                        time: "1970-01-01T00:00:00Z".to_string(),
                        location: vec![52.4862, 13.45148].to_loc(),
                    },
                    end: Some(VehiclePlace {
                        time: "1970-01-01T00:01:40Z".to_string(),
                        location: vec![52.4862, 13.45148].to_loc(),
                    }),
                    breaks: Some(vec![VehicleBreak {
                        time: VehicleBreakTime::TimeWindow(vec![
                            "1970-01-01T00:00:10Z".to_string(),
                            "1970-01-01T00:01:20Z".to_string(),
                        ]),
                        duration: 100.0,
                        locations: Some(vec![vec![52.48315, 13.4330].to_loc()]),
                    }]),
                    reloads: None,
                }],
                capacity: vec![10, 1],
                skills: Some(vec!["unique1".to_string(), "unique2".to_string()]),
                limits: Some(VehicleLimits { max_distance: Some(123.1), shift_time: Some(100.), allowed_areas: None }),
            }],
            profiles: create_default_profiles(),
        },
        objectives: None,
        config: None,
    };
    let matrix = Matrix {
        profile: "car".to_owned(),
        timestamp: None,
        travel_times: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        distances: vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        error_codes: Option::None,
    };

    let problem = (problem, vec![matrix]).read_pragmatic().ok().unwrap();

    assert_eq!(problem.jobs.all().collect::<Vec<_>>().len(), 3 + 2);

    // delivery
    let job = get_single_job(0, problem.jobs.as_ref());
    let place = get_single_place(job.as_ref());
    assert_eq!(job.dimens.get_id().unwrap(), "delivery_job");
    assert_eq!(place.duration, 100.);
    assert_eq!(place.location.unwrap(), 0);
    assert_demand(
        job.dimens.get_demand().unwrap(),
        &Demand {
            pickup: (MultiDimensionalCapacity::default(), MultiDimensionalCapacity::default()),
            delivery: (MultiDimensionalCapacity::new(vec![0, 1]), MultiDimensionalCapacity::default()),
        },
    );
    assert_time_spans(&place.times, vec![(0., 100.), (110., 120.)]);
    assert_skills(&job.dimens, Some(vec!["unique".to_string()]));

    // shipment
    let job = get_multi_job(1, problem.jobs.as_ref());
    assert_eq!(job.dimens.get_id().unwrap(), "pickup_delivery_job");
    assert_skills(&job.dimens, None);

    let pickup = job.jobs.first().unwrap().clone();
    let place = get_single_place(pickup.as_ref());
    assert_eq!(place.duration, 110.);
    assert_eq!(place.location.unwrap(), 1);
    assert_demand(pickup.dimens.get_demand().unwrap(), &single_demand_as_multi((0, 2), (0, 0)));
    assert_time_spans(&place.times, vec![(10., 30.)]);

    let delivery = job.jobs.last().unwrap().clone();
    let place = get_single_place(delivery.as_ref());
    assert_eq!(place.duration, 120.);
    assert_eq!(place.location.unwrap(), 0);
    assert_demand(delivery.dimens.get_demand().unwrap(), &single_demand_as_multi((0, 0), (0, 2)));
    assert_time_spans(&place.times, vec![(50., 60.)]);

    // pickup
    let job = get_single_job(2, problem.jobs.as_ref());
    let place = get_single_place(job.as_ref());
    assert_eq!(job.dimens.get_id().unwrap(), "pickup_job");
    assert_eq!(place.duration, 90.);
    assert_eq!(place.location.unwrap(), 2);
    assert_demand(job.dimens.get_demand().unwrap(), &single_demand_as_multi((3, 0), (0, 0)));
    assert_time_spans(&place.times, vec![(10., 70.)]);
    assert_skills(&job.dimens, Some(vec!["unique2".to_string()]));

    // fleet
    assert_eq!(problem.fleet.profiles.len(), 1);
    assert_eq!(problem.fleet.drivers.len(), 1);
    assert_eq!(problem.fleet.vehicles.len(), 2);

    (1..3).for_each(|index| {
        let vehicle = problem.fleet.vehicles.get(index - 1).unwrap();
        assert_eq!(*vehicle.dimens.get_id().unwrap(), format!("my_vehicle_{}", index));
        assert_eq!(vehicle.profile, 0);
        assert_eq!(vehicle.costs.fixed, 100.0);
        assert_eq!(vehicle.costs.per_distance, 1.0);
        assert_eq!(vehicle.costs.per_driving_time, 2.0);
        assert_eq!(vehicle.costs.per_waiting_time, 2.0);
        assert_eq!(vehicle.costs.per_service_time, 2.0);

        assert_eq!(vehicle.details.len(), 1);
        let detail = vehicle.details.first().unwrap();
        assert_eq!(detail.start.unwrap(), 3);
        assert_eq!(detail.end.unwrap(), 3);
        assert_time_window(detail.time.as_ref().unwrap(), &(0., 100.));
        assert_skills(&vehicle.dimens, Some(vec!["unique1".to_string(), "unique2".to_string()]));
    });
}

#[test]
fn can_deserialize_minimal_problem_and_matrix() {
    let problem = (SIMPLE_PROBLEM.to_string(), vec![SIMPLE_MATRIX.to_string()]).read_pragmatic().ok().unwrap();

    assert_eq!(problem.fleet.vehicles.len(), 1);
    assert_eq!(problem.jobs.all().collect::<Vec<_>>().len(), 2);
    assert!(problem.locks.is_empty());

    assert_time_window(
        problem.fleet.vehicles.first().as_ref().unwrap().details.first().as_ref().unwrap().time.as_ref().unwrap(),
        &(1562230800., 1562263200.),
    );
}

#[test]
fn can_create_approximation_matrices() {
    let problem = Problem {
        plan: Plan {
            jobs: vec![
                create_delivery_job("job1", vec![52.52599, 13.45413]),
                create_delivery_job("job2", vec![52.5165, 13.3808]),
            ],
            relations: None,
        },
        fleet: Fleet {
            vehicles: vec![],
            profiles: vec![
                Profile { name: "car1".to_string(), profile_type: "car".to_string(), speed: Some(8.) },
                Profile { name: "car2".to_string(), profile_type: "car".to_string(), speed: Some(10.) },
                Profile { name: "car3".to_string(), profile_type: "car".to_string(), speed: Some(5.) },
                Profile { name: "car4".to_string(), profile_type: "car".to_string(), speed: None },
            ],
        },
        ..create_empty_problem()
    };

    let matrices = create_approx_matrices(&problem);
    assert_eq!(matrices.len(), 4);

    for &(profile, duration) in &[("car1", 635), ("car2", 508), ("car3", 1016), ("car4", 508)] {
        let matrix = matrices.iter().find(|m| m.profile.as_str() == profile).unwrap();

        assert!(matrix.error_codes.is_none());
        assert!(matrix.timestamp.is_none());

        assert_eq!(matrix.distances, &[0, 5078, 5078, 0]);
        assert_eq!(matrix.travel_times, &[0, duration, duration, 0]);
    }
}
