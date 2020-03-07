use super::*;
use crate::validation::common::get_time_windows;
use std::ops::Deref;
use vrp_core::models::common::TimeWindow;

/// Checks that fleet has no vehicle with duplicate type ids.
fn check_e1003_no_vehicle_types_with_duplicate_type_ids(ctx: &ValidationContext) -> Result<(), String> {
    get_duplicates(ctx.vehicles().map(|vehicle| &vehicle.type_id))
        .map_or(Ok(()), |ids| Err(format!("E1003: Duplicated vehicle type ids: {}", ids.join(", "))))
}

/// Checks that fleet has no vehicle with duplicate ids.
fn check_e1004_no_vehicle_types_with_duplicate_ids(ctx: &ValidationContext) -> Result<(), String> {
    get_duplicates(ctx.vehicles().flat_map(|vehicle| vehicle.vehicle_ids.iter()))
        .map_or(Ok(()), |ids| Err(format!("E1004: Duplicated vehicle ids: {}", ids.join(", "))))
}

/// Checks that vehicle shift time is correct.
fn check_e1005_vehicle_shift_time(ctx: &ValidationContext) -> Result<(), String> {
    let type_ids = ctx
        .vehicles()
        .filter_map(|vehicle| {
            let tws = vehicle
                .shifts
                .iter()
                .map(|shift| {
                    vec![
                        shift.start.time.clone(),
                        shift.end.as_ref().map_or_else(|| shift.start.time.clone(), |end| end.time.clone()),
                    ]
                })
                .collect::<Vec<_>>();
            if check_raw_time_windows(&tws, false) {
                None
            } else {
                Some(vehicle.type_id.to_string())
            }
        })
        .collect::<Vec<_>>();

    if type_ids.is_empty() {
        Ok(())
    } else {
        Err(format!("E1005: Invalid start or end times in vehicle shifts: {}", type_ids.join(", ")))
    }
}

/// Checks that break time window is correct.
fn check_e1006_vehicle_breaks_time_is_correct(ctx: &ValidationContext) -> Result<(), String> {
    let type_ids = get_invalid_type_ids(
        ctx,
        Box::new(|shift, shift_time| {
            shift
                .breaks
                .as_ref()
                .map(|breaks| {
                    let tws = breaks
                        .iter()
                        .filter_map(|b| match &b.times {
                            VehicleBreakTime::TimeWindows(tws) => Some(get_time_windows(tws)),
                            _ => None,
                        })
                        .flatten()
                        .collect::<Vec<_>>();

                    check_shift_time_windows(shift_time, tws, false)
                })
                .unwrap_or(true)
        }),
    );

    if type_ids.is_empty() {
        Ok(())
    } else {
        Err(format!("E1006: Invalid break time windows in vehicle shifts: {}", type_ids.join(", ")))
    }
}

/// Checks that reload time windows are correct.
fn check_e1007_vehicle_reload_time_is_correct(ctx: &ValidationContext) -> Result<(), String> {
    let type_ids = get_invalid_type_ids(
        ctx,
        Box::new(|shift, shift_time| {
            shift
                .reloads
                .as_ref()
                .map(|reloads| {
                    let tws = reloads
                        .iter()
                        .filter_map(|reload| reload.times.as_ref())
                        .map(|tws| get_time_windows(tws))
                        .flatten()
                        .collect::<Vec<_>>();

                    check_shift_time_windows(shift_time, tws, true)
                })
                .unwrap_or(true)
        }),
    );

    if type_ids.is_empty() {
        Ok(())
    } else {
        Err(format!("E1007: Invalid reload time windows in vehicle shifts: {}", type_ids.join(", ")))
    }
}

fn get_invalid_type_ids(
    ctx: &ValidationContext,
    check_shift: Box<dyn Fn(&VehicleShift, Option<TimeWindow>) -> bool>,
) -> Vec<String> {
    ctx.vehicles()
        .filter_map(|vehicle| {
            let all_correct =
                vehicle.shifts.iter().all(|shift| check_shift.deref()(shift, get_shift_time_window(shift)));

            if all_correct {
                None
            } else {
                Some(vehicle.type_id.clone())
            }
        })
        .collect::<Vec<_>>()
}

fn check_shift_time_windows(
    shift_time: Option<TimeWindow>,
    tws: Vec<Option<TimeWindow>>,
    skip_intersection_check: bool,
) -> bool {
    tws.is_empty()
        || (check_time_windows(&tws, skip_intersection_check)
            && shift_time
                .as_ref()
                .map_or(true, |shift_time| tws.into_iter().map(|tw| tw.unwrap()).all(|tw| tw.intersects(shift_time))))
}

fn get_shift_time_window(shift: &VehicleShift) -> Option<TimeWindow> {
    get_time_window(
        &shift.start.time,
        &shift.end.clone().map_or_else(|| "2200-07-04T00:00:00Z".to_string(), |end| end.time),
    )
}

/// Validates vehicles from the fleet.
pub fn validate_vehicles(ctx: &ValidationContext) -> Result<(), Vec<String>> {
    let errors = check_e1003_no_vehicle_types_with_duplicate_type_ids(ctx)
        .err()
        .iter()
        .cloned()
        .chain(check_e1004_no_vehicle_types_with_duplicate_ids(ctx).err().iter().cloned())
        .chain(check_e1005_vehicle_shift_time(ctx).err().iter().cloned())
        .chain(check_e1006_vehicle_breaks_time_is_correct(ctx).err().iter().cloned())
        .chain(check_e1007_vehicle_reload_time_is_correct(ctx).err().iter().cloned())
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
