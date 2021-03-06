#[cfg(test)]
#[path = "../../../tests/unit/solver/termination/cost_variation_test.rs"]
mod cost_variation_test;

use crate::models::common::{Cost, Objective};
use crate::solver::termination::Termination;
use crate::solver::RefinementContext;
use crate::utils::get_cv;

/// Stops when maximum amount of generations is exceeded.
pub struct CostVariation {
    sample: usize,
    threshold: f64,
    key: String,
}

impl CostVariation {
    /// Creates a new instance of [`CostVariation`].
    pub fn new(sample: usize, threshold: f64) -> Self {
        Self { sample, threshold, key: "coeff_var".to_string() }
    }

    fn update_and_check(&self, refinement_ctx: &mut RefinementContext, cost: Cost) -> bool {
        let costs = refinement_ctx
            .state
            .entry(self.key.clone())
            .or_insert_with(|| Box::new(vec![0.; self.sample]))
            .downcast_mut::<Vec<f64>>()
            .unwrap();

        costs[refinement_ctx.generation % self.sample] = cost;

        refinement_ctx.generation >= (self.sample - 1) && self.check_threshold(costs)
    }

    fn check_threshold(&self, costs: &[f64]) -> bool {
        get_cv(costs) < self.threshold
    }
}

impl Termination for CostVariation {
    fn is_termination(&self, refinement_ctx: &mut RefinementContext) -> bool {
        if let Some(best) = refinement_ctx.population.best() {
            let cost = refinement_ctx.problem.objective.fitness(best);
            self.update_and_check(refinement_ctx, cost)
        } else {
            false
        }
    }
}
