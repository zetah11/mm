use std::collections::HashMap;

use rational::Rational;

use crate::Length;

use super::equation::{Equation, Term, Variable};
use super::matrix::{solve, Row, System};
use super::Checker;

impl Checker<'_> {
    pub fn solve(&mut self, equations: Vec<Equation>) {
        let vars: Vec<_> = equations.iter().map(|eq| eq.var).collect();
        let var_positions: HashMap<_, _> = vars
            .iter()
            .enumerate()
            .map(|(index, var)| (*var, index))
            .collect();

        // Variables are in the same order as the equation list
        let rows = equations
            .into_iter()
            .map(|equation| self.make_row(&var_positions, equation))
            .collect();

        let system = System::new(rows, vars);
        let solution = match solve(system) {
            Some(solution) => solution,
            None => todo!(),
        };

        for (var, length) in solution {
            debug_assert!(self.lengths.insert(var, Length(length)).is_none());
        }
    }

    fn make_row(&self, var_positions: &HashMap<Variable, usize>, mut equation: Equation) -> Row {
        if equation.sums.len() != 1 {
            todo!()
        }

        let mut constant = Rational::zero();
        let mut coeffs = vec![Rational::zero(); var_positions.len()];

        for term in equation.sums.remove(0).terms {
            match term {
                Term::Constant(value) => constant += value.0,
                Term::Variable(factor, var) => {
                    if let Some(pos) = var_positions.get(&var) {
                        coeffs[*pos] += factor.0;
                    } else {
                        let length = self
                            .lengths
                            .get(&var)
                            .expect("melodies are processed in topological order");
                        constant += length.0;
                    }
                }
            }
        }

        Row { coeffs, constant }
    }
}
