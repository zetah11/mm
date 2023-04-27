use std::collections::HashMap;

use itertools::Itertools;
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
        let possible_rows = equations
            .into_iter()
            .map(|equation| self.make_row(&var_positions, equation))
            .multi_cartesian_product();

        let mut result = vec![Rational::zero(); vars.len()];

        for rows in possible_rows {
            let system = System::new(rows, vars.clone());
            match solve(system) {
                Some(solution) => {
                    for (total, (_, new)) in result.iter_mut().zip(solution) {
                        *total = (*total).max(new);
                    }
                }

                None => todo!(),
            };
        }

        for (var, length) in vars.into_iter().zip(result) {
            debug_assert!(self.lengths.insert(var, Length(length)).is_none());
        }
    }

    fn make_row(&self, var_positions: &HashMap<Variable, usize>, equation: Equation) -> Vec<Row> {
        let index = *var_positions
            .get(&equation.var)
            .expect("equation variable is part of the equation");

        equation
            .sums
            .into_iter()
            .map(|sum| {
                let mut constant = Rational::zero();
                let mut coeffs = vec![Rational::zero(); var_positions.len()];

                coeffs[index] = Rational::one();

                for term in sum.terms {
                    match term {
                        Term::Constant(value) => constant += value.0,
                        Term::Variable(factor, var) => {
                            if let Some(pos) = var_positions.get(&var) {
                                coeffs[*pos] -= factor.0;
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
            })
            .collect()
    }
}
