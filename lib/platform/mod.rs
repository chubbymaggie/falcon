//! Platform models an external system like Linux or CGC.

use engine::engine::*;
use error::*;
use il;

pub mod linux;
pub mod linux_x86;

/// Platform provides generic interaction between Falcon and a modelled system.
pub trait Platform<P: Platform<P>> : Clone {
    /// Handle an `Operation::Raise` from a `SymbolicEngine`. Returns a vec of tuples of produced
    /// `(Platform, SymbolicEngine)`.
    fn raise(self, expression: &il::Expression, engine: SymbolicEngine)
    -> Result<Vec<(P, SymbolicEngine)>>;

    /// Get each `Scalar` produced by this `Platform`.
    fn symbolic_variables(&self) -> Vec<il::Scalar>;
}