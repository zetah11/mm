use std::fmt;

use rational::Rational;

/// Represents a system of linear equations in terms of an augmented matrix of
/// rows multiplied by a vector of variables.
#[derive(Eq, PartialEq)]
pub struct System<V> {
    rows: Vec<Row>,
    vars: Vec<V>,
    size: usize,
}

impl<V> System<V> {
    pub fn new(rows: Vec<Row>, vars: Vec<V>) -> Self {
        let size = vars.len();

        debug_assert!(rows.len() == size);
        debug_assert!(rows.iter().all(|row| row.coeffs.len() == size));

        Self { rows, vars, size }
    }

    /// Returns `true` if this system is in row-echelon form.
    fn is_row_echelon(&self) -> bool {
        let mut rows = self.rows.iter();
        let mut zeroes = if let Some(first) = rows.next() {
            first.zeroes()
        } else {
            return true;
        };

        for row in rows {
            let row_zeroes = row.zeroes();
            if row_zeroes <= zeroes {
                return false;
            }
            zeroes = row_zeroes;
        }

        true
    }
}

impl<V> fmt::Debug for System<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut rows = self.rows.iter();

        if let Some(row) = rows.next() {
            write!(f, "{row:?}")?;
        }

        for row in rows {
            write!(f, ", {row:?}")?;
        }

        write!(f, "]")
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Row {
    pub coeffs: Vec<Rational>,
    pub constant: Rational,
}

impl Row {
    fn zeroes(&self) -> usize {
        let mut zeroes = 0;
        for coeff in self.coeffs.iter().copied() {
            if coeff != Rational::zero() {
                break;
            }

            zeroes += 1;
        }
        zeroes
    }
}

impl fmt::Debug for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut coeffs = self.coeffs.iter();

        if let Some(coeff) = coeffs.next() {
            write!(f, "{coeff}")?;
        }

        for coeff in coeffs {
            write!(f, ", {coeff}")?;
        }

        write!(f, " | {}]", self.constant)
    }
}

/// Attempt to solve the given system of linear equations. Returns `None` if
/// the system has no solution.
pub fn solve<V>(mut system: System<V>) -> Option<Vec<(V, Rational)>> {
    eliminate(&mut system)?;
    backsolve(system)
}

/// Solve a system of linear equations through backsubstitution, assuming the
/// system is in row echelon form.
fn backsolve<V>(system: System<V>) -> Option<Vec<(V, Rational)>> {
    debug_assert!(system.is_row_echelon());

    let mut solutions = vec![Rational::zero(); system.size];

    for (index, row) in system.rows.iter().enumerate().rev() {
        let mut sum = row.constant;

        for (col, solution) in row.coeffs.iter().zip(solutions.iter()).skip(index + 1) {
            sum -= *col * *solution;
        }

        solutions[index] = sum / row.coeffs[index];
    }

    Some(system.vars.into_iter().zip(solutions).collect())
}

/// Perform Gaussian elimination on the system of equations.
fn eliminate<V>(system: &mut System<V>) -> Option<()> {
    for column in 0..system.size {
        if let Some(largest) = largest_row_index(system, column, column) {
            system.rows.swap(column, largest);
        } else {
            return None;
        }

        let big = system.rows[column].coeffs[column];
        for row in column + 1..system.size {
            let scale = system.rows[row].coeffs[column] / big;

            for col in column + 1..system.size {
                let above = system.rows[column].coeffs[col];
                system.rows[row].coeffs[col] -= scale * above;
            }

            system.rows[row].coeffs[column] = Rational::zero();

            let above = system.rows[column].constant;
            system.rows[row].constant -= scale * above;
        }
    }

    Some(())
}

/// Get the index of the row with the largest value at the given column,
/// ignoring rows before `from`. Returns `None` if the largest value is `0`.
fn largest_row_index<V>(system: &System<V>, column: usize, from: usize) -> Option<usize> {
    debug_assert!(column < system.size);

    let mut max = Rational::zero();
    let mut at = None;

    for (index, row) in system.rows.iter().enumerate().skip(from) {
        let value = row.coeffs[column].abs();
        if value > max {
            max = value;
            at = Some(index);
        }
    }

    if max == Rational::zero() {
        None
    } else {
        at
    }
}

#[cfg(test)]
mod tests {
    use rational::Rational;

    use crate::check::matrix::{largest_row_index, solve};

    use super::{eliminate, Row, System};

    #[test]
    fn argmax() {
        /*
        [  0  1  2 |  3 ]
        [  4  9  6 | 11 ]
        [  8  5 10 |  7 ]
        */

        let r1 = Row {
            coeffs: vec![
                Rational::integer(0),
                Rational::integer(1),
                Rational::integer(2),
            ],
            constant: Rational::integer(3),
        };

        let r2 = Row {
            coeffs: vec![
                Rational::integer(4),
                Rational::integer(9),
                Rational::integer(6),
            ],
            constant: Rational::integer(11),
        };

        let r3 = Row {
            coeffs: vec![
                Rational::integer(8),
                Rational::integer(5),
                Rational::integer(10),
            ],
            constant: Rational::integer(7),
        };

        let rows = vec![r1, r2, r3];
        let vars = vec!["a", "b", "c"];
        let system = System::new(rows, vars);

        assert_eq!(Some(2), largest_row_index(&system, 0, 0));
        assert_eq!(Some(1), largest_row_index(&system, 1, 1));
        assert_eq!(Some(2), largest_row_index(&system, 2, 2));
    }

    #[test]
    fn eliminate_2x2() {
        /*
        [  3  4 | 33 ]
        [  5 -2 |  3 ]
        */

        let r1 = Row {
            coeffs: vec![Rational::integer(3), Rational::integer(4)],
            constant: Rational::integer(33),
        };

        let r2 = Row {
            coeffs: vec![Rational::integer(5), Rational::integer(-2)],
            constant: Rational::integer(3),
        };

        let rows = vec![r1, r2];
        let vars = vec!["a", "b"];
        let mut system = System::new(rows, vars);

        eliminate(&mut system);

        let r1 = Row {
            coeffs: vec![Rational::integer(5), Rational::integer(-2)],
            constant: Rational::integer(3),
        };

        let r2 = Row {
            coeffs: vec![Rational::zero(), Rational::new(26, 5)],
            constant: Rational::new(156, 5),
        };

        let rows = vec![r1, r2];
        let vars = vec!["a", "b"];
        let expected = System::new(rows, vars);

        assert_eq!(expected, system);
    }

    #[test]
    fn eliminate_3x3() {
        /*
        [  1  3  7 | 18 ]
        [  3  6  9 | 33 ]
        [  3  9 15 | 48 ]
        */

        let r1 = Row {
            coeffs: vec![
                Rational::integer(1),
                Rational::integer(3),
                Rational::integer(7),
            ],
            constant: Rational::integer(18),
        };

        let r2 = Row {
            coeffs: vec![
                Rational::integer(3),
                Rational::integer(6),
                Rational::integer(9),
            ],
            constant: Rational::integer(33),
        };

        let r3 = Row {
            coeffs: vec![
                Rational::integer(3),
                Rational::integer(9),
                Rational::integer(15),
            ],
            constant: Rational::integer(48),
        };

        let rows = vec![r1, r2, r3];
        let vars = vec!["x", "y", "z"];
        let mut system = System::new(rows, vars);

        eliminate(&mut system);

        let r1 = Row {
            coeffs: vec![
                Rational::integer(3),
                Rational::integer(6),
                Rational::integer(9),
            ],
            constant: Rational::integer(33),
        };

        let r2 = Row {
            coeffs: vec![Rational::zero(), Rational::integer(3), Rational::integer(6)],
            constant: Rational::integer(15),
        };

        let r3 = Row {
            coeffs: vec![Rational::zero(), Rational::zero(), Rational::integer(2)],
            constant: Rational::integer(2),
        };

        let rows = vec![r1, r2, r3];
        let vars = vec!["x", "y", "z"];
        let expected = System::new(rows, vars);

        assert_eq!(expected, system);
    }

    #[test]
    fn solve_2x2() {
        /*
        [  3  4 | 33 ]
        [  5 -2 |  3 ]
        */

        let r1 = Row {
            coeffs: vec![Rational::integer(3), Rational::integer(4)],
            constant: Rational::integer(33),
        };

        let r2 = Row {
            coeffs: vec![Rational::integer(5), Rational::integer(-2)],
            constant: Rational::integer(3),
        };

        let rows = vec![r1, r2];
        let vars = vec!["a", "b"];
        let system = System::new(rows, vars);

        let expected = Some(vec![
            ("a", Rational::integer(3)),
            ("b", Rational::integer(6)),
        ]);

        let actual = solve(system);
        assert_eq!(expected, actual);
    }

    #[test]
    fn solve_3x3() {
        /*
        [  1  3  7 | 18 ]
        [  3  6  9 | 33 ]
        [  3  9 15 | 48 ]
        */

        let r1 = Row {
            coeffs: vec![
                Rational::integer(1),
                Rational::integer(3),
                Rational::integer(7),
            ],
            constant: Rational::integer(18),
        };

        let r2 = Row {
            coeffs: vec![
                Rational::integer(3),
                Rational::integer(6),
                Rational::integer(9),
            ],
            constant: Rational::integer(33),
        };

        let r3 = Row {
            coeffs: vec![
                Rational::integer(3),
                Rational::integer(9),
                Rational::integer(15),
            ],
            constant: Rational::integer(48),
        };

        let rows = vec![r1, r2, r3];
        let vars = vec!["x", "y", "z"];
        let system = System::new(rows, vars);

        let expected = Some(vec![
            ("x", Rational::integer(2)),
            ("y", Rational::integer(3)),
            ("z", Rational::integer(1)),
        ]);

        let actual = solve(system);

        assert_eq!(expected, actual);
    }

    #[test]
    fn unsolvable_1x1() {
        /*
        [ 0 | 5 ]
        */

        let r1 = Row {
            coeffs: vec![Rational::zero()],
            constant: Rational::integer(5),
        };

        let rows = vec![r1];
        let vars = vec!["n"];
        let system = System::new(rows, vars);

        let expected = None;
        let actual = solve(system);

        assert_eq!(expected, actual);
    }
}
