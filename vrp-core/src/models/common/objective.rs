use std::cmp::Ordering;

/// An *objective* defines a *total ordering relation* and a *distance metric* on a set of
/// `solutions`. Given any two solutions, an objective answers the following two questions:
///
/// - "which solution is the better one" (total order)
/// - "how similar are the two solutions" (distance metric)
pub trait Objective {
    /// The solution value type that we define the objective on.
    type Solution;

    /// An objective defines a total ordering between any two solution values.
    ///
    /// This answers the question, is solution `a` better, equal or worse than solution `b`,
    /// according to the objective.
    fn total_order(&self, a: &Self::Solution, b: &Self::Solution) -> Ordering;

    /// An objective defines a distance metric between any two solution values.
    ///
    /// The distance metric answer the question, how similar the solutions `a` and `b` are,
    /// according to the objective. A zero value would mean, that both solutions are in fact the same,
    /// according to the objective. Larger magnitudes would mean "less similar".
    fn distance(&self, a: &Self::Solution, b: &Self::Solution) -> f64;

    /// An objective fitness value for given `solution`.
    fn fitness(&self, solution: &Self::Solution) -> f64;
}

/// A multi objective.
pub trait MultiObjective: Objective {
    fn objectives<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &Box<dyn Objective<Solution = Self::Solution> + Send + Sync>> + 'a>;
}
