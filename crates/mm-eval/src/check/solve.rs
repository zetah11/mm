use std::collections::HashMap;

use itertools::Itertools;
use num_bigint::BigInt;
use num_rational::BigRational;

use crate::span::Span;
use crate::{melody, Allocator, Length};

use super::equation::{Equation, Term, Variable};
use super::matrix::{self, solve, Row, System};
use super::{Checker, Error};

enum Solution {
    Solved(Vec<(Variable, BigRational)>),
    Unbounded(Vec<Variable>),
    Unfounded(Vec<Variable>),
}

impl<N, Id, A: Allocator<melody::Melody<N, Id, A>>> Checker<'_, N, Id, A> {
    pub fn solve(&mut self, equations: Vec<Equation>, span: Span<Id>) {
        match self.solve_equations(equations) {
            Solution::Solved(lengths) => {
                for (var, length) in lengths {
                    let prev = self.lengths.insert(var, Length::Bounded(length));
                    debug_assert!(prev.is_none());
                }
            }

            Solution::Unbounded(vars) => {
                for var in vars {
                    let prev = self.lengths.insert(var, Length::Unbounded);
                    debug_assert!(prev.is_none());
                }
            }

            Solution::Unfounded(vars) => {
                self.errors.push(Error::UnfoundedRecursion(span));
                for var in vars {
                    let prev = self.lengths.insert(var, Length::zero());
                    debug_assert!(prev.is_none());
                }
            }
        }
    }

    fn solve_equations(&mut self, equations: Vec<Equation>) -> Solution {
        let vars: Vec<_> = equations.iter().map(|eq| eq.var).collect();
        let var_positions: HashMap<_, _> = vars
            .iter()
            .enumerate()
            .map(|(index, var)| (*var, index))
            .collect();

        let mut possible_rows = Vec::with_capacity(equations.len());
        for equation in equations {
            match self.make_row(&var_positions, equation) {
                Some(rows) => possible_rows.push(rows),
                None => return Solution::Unbounded(vars),
            }
        }

        // Variables are in the same order as the equation list
        let possible_rows = possible_rows.into_iter().multi_cartesian_product();

        let mut result = vec![BigRational::from_integer(BigInt::from(0)); vars.len()];

        for rows in possible_rows {
            let system = System::new(rows, vars.clone());
            match solve(system) {
                Ok(solution) => {
                    for (total, (_, mut new)) in result.iter_mut().zip(solution) {
                        *total = total.max(&mut new).clone();
                    }
                }

                Err(matrix::Error::Contradiction) => {
                    return Solution::Unbounded(vars);
                }

                Err(matrix::Error::Unfounded) => {
                    return Solution::Unfounded(vars);
                }
            };
        }

        Solution::Solved(vars.into_iter().zip(result).collect())
    }

    fn make_row(
        &self,
        var_positions: &HashMap<Variable, usize>,
        equation: Equation,
    ) -> Option<Vec<Row>> {
        let index = *var_positions
            .get(&equation.var)
            .expect("equation variable is part of the equation");

        let mut rows = Vec::with_capacity(equation.sums.len());
        for sum in equation.sums {
            let mut constant = BigRational::from_integer(BigInt::from(0));
            let mut coeffs = vec![BigRational::from_integer(BigInt::from(0)); var_positions.len()];

            coeffs[index] = BigRational::from_integer(BigInt::from(1));

            for term in sum.terms {
                match term {
                    Term::Constant(Length::Bounded(length)) => constant += length,
                    Term::Constant(Length::Unbounded) => return None,

                    Term::Variable(factor, var) => {
                        if let Some(pos) = var_positions.get(&var) {
                            coeffs[*pos] -= factor.0;
                        } else {
                            let length = self
                                .lengths
                                .get(&var)
                                .expect("melodies are processed in topological order");

                            let Length::Bounded(length) = &factor * length else { return None; };
                            constant += length;
                        }
                    }
                }
            }

            rows.push(Row { coeffs, constant });
        }

        Some(rows)
    }
}
