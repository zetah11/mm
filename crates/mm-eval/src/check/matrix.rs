use std::fmt;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Signed;

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
    pub coeffs: Vec<BigRational>,
    pub constant: BigRational,
}

impl Row {
    fn zeroes(&self) -> usize {
        let mut zeroes = 0;
        let zero = BigRational::from_integer(BigInt::from(0));
        for coeff in self.coeffs.iter() {
            if coeff != &zero {
                break;
            }

            zeroes += 1;
        }
        zeroes
    }

    fn all_zeroes(&self) -> bool {
        let zero = BigRational::from_integer(BigInt::from(0));
        self.zeroes() == self.coeffs.len() && self.constant == zero
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// The system contains a contradiction.
    Contradiction,
    /// The system is underspecified.
    Unfounded,
}

/// Attempt to solve the given system of linear equations. Returns `None` if
/// the system has no solution.
pub fn solve<V>(mut system: System<V>) -> Result<Vec<(V, BigRational)>, Error> {
    eliminate(&mut system)?;
    Ok(backsolve(system))
}

/// Solve a system of linear equations through backsubstitution, assuming the
/// system is in row echelon form.
fn backsolve<V>(system: System<V>) -> Vec<(V, BigRational)> {
    debug_assert!(system.is_row_echelon());

    let mut solutions = vec![BigRational::from_integer(BigInt::from(0)); system.size];

    for (index, row) in system.rows.iter().enumerate().rev() {
        let mut sum = row.constant.clone();

        for (col, solution) in row.coeffs.iter().zip(solutions.iter()).skip(index + 1) {
            sum -= col.clone() * solution.clone();
        }

        solutions[index] = sum / row.coeffs[index].clone();
    }

    system.vars.into_iter().zip(solutions).collect()
}

/// Perform Gaussian elimination on the system of equations.
fn eliminate<V>(system: &mut System<V>) -> Result<(), Error> {
    for column in 0..system.size {
        if let Some(largest) = largest_row_index(system, column, column) {
            system.rows.swap(column, largest);
        } else {
            // If a row consists of all zeroes, then *technically* there's no
            // solution, but practically, we want every variable to solve to
            // zero.
            if system.rows.iter().any(|row| row.all_zeroes()) {
                return Err(Error::Unfounded);
            } else {
                return Err(Error::Contradiction);
            }
        }

        let big = system.rows[column].coeffs[column].clone();
        for row in column + 1..system.size {
            let scale = system.rows[row].coeffs[column].clone() / big.clone();

            for col in column + 1..system.size {
                let above = system.rows[column].coeffs[col].clone();
                system.rows[row].coeffs[col] -= scale.clone() * above;
            }

            system.rows[row].coeffs[column] = BigRational::from_integer(BigInt::from(0));

            let above = system.rows[column].constant.clone();
            system.rows[row].constant -= scale * above;
        }
    }

    Ok(())
}

/// Get the index of the row with the largest value at the given column,
/// ignoring rows before `from`. Returns `None` if the largest value is `0`.
fn largest_row_index<V>(system: &System<V>, column: usize, from: usize) -> Option<usize> {
    debug_assert!(column < system.size);

    let mut max = BigRational::from_integer(BigInt::from(0));
    let mut at = None;

    for (index, row) in system.rows.iter().enumerate().skip(from) {
        let value = row.coeffs[column].abs();
        if value > max {
            max = value;
            at = Some(index);
        }
    }

    if max == BigRational::from_integer(BigInt::from(0)) {
        None
    } else {
        at
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::{eliminate, Error, Row, System};
    use crate::check::matrix::{largest_row_index, solve};

    fn r(n: i128, d: i128) -> BigRational {
        BigRational::new(BigInt::from(n), BigInt::from(d))
    }

    #[test]
    fn argmax() {
        /*
        [  0  1  2 |  3 ]
        [  4  9  6 | 11 ]
        [  8  5 10 |  7 ]
        */

        let r1 = Row {
            coeffs: vec![r(0, 1), r(1, 1), r(2, 1)],
            constant: r(3, 1),
        };

        let r2 = Row {
            coeffs: vec![r(4, 1), r(9, 1), r(6, 1)],
            constant: r(11, 1),
        };

        let r3 = Row {
            coeffs: vec![r(8, 1), r(5, 1), r(10, 1)],
            constant: r(7, 1),
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
            coeffs: vec![r(3, 1), r(4, 1)],
            constant: r(33, 1),
        };

        let r2 = Row {
            coeffs: vec![r(5, 1), r(-2, 1)],
            constant: r(3, 1),
        };

        let rows = vec![r1, r2];
        let vars = vec!["a", "b"];
        let mut system = System::new(rows, vars);

        eliminate(&mut system).unwrap();

        let r1 = Row {
            coeffs: vec![r(5, 1), r(-2, 1)],
            constant: r(3, 1),
        };

        let r2 = Row {
            coeffs: vec![r(0, 1), r(26, 5)],
            constant: r(156, 5),
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
            coeffs: vec![r(1, 1), r(3, 1), r(7, 1)],
            constant: r(18, 1),
        };

        let r2 = Row {
            coeffs: vec![r(3, 1), r(6, 1), r(9, 1)],
            constant: r(33, 1),
        };

        let r3 = Row {
            coeffs: vec![r(3, 1), r(9, 1), r(15, 1)],
            constant: r(48, 1),
        };

        let rows = vec![r1, r2, r3];
        let vars = vec!["x", "y", "z"];
        let mut system = System::new(rows, vars);

        eliminate(&mut system).unwrap();

        let r1 = Row {
            coeffs: vec![r(3, 1), r(6, 1), r(9, 1)],
            constant: r(33, 1),
        };

        let r2 = Row {
            coeffs: vec![r(0, 1), r(3, 1), r(6, 1)],
            constant: r(15, 1),
        };

        let r3 = Row {
            coeffs: vec![r(0, 1), r(0, 1), r(2, 1)],
            constant: r(2, 1),
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
            coeffs: vec![r(3, 1), r(4, 1)],
            constant: r(33, 1),
        };

        let r2 = Row {
            coeffs: vec![r(5, 1), r(-2, 1)],
            constant: r(3, 1),
        };

        let rows = vec![r1, r2];
        let vars = vec!["a", "b"];
        let system = System::new(rows, vars);

        let expected = Ok(vec![("a", r(3, 1)), ("b", r(6, 1))]);

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
            coeffs: vec![r(1, 1), r(3, 1), r(7, 1)],
            constant: r(18, 1),
        };

        let r2 = Row {
            coeffs: vec![r(3, 1), r(6, 1), r(9, 1)],
            constant: r(33, 1),
        };

        let r3 = Row {
            coeffs: vec![r(3, 1), r(9, 1), r(15, 1)],
            constant: r(48, 1),
        };

        let rows = vec![r1, r2, r3];
        let vars = vec!["x", "y", "z"];
        let system = System::new(rows, vars);

        let expected = Ok(vec![("x", r(2, 1)), ("y", r(3, 1)), ("z", r(1, 1))]);

        let actual = solve(system);

        assert_eq!(expected, actual);
    }

    #[test]
    fn unsolvable_1x1() {
        /*
        [ 0 | 5 ]
        */

        let r1 = Row {
            coeffs: vec![r(0, 1)],
            constant: r(5, 1),
        };

        let rows = vec![r1];
        let vars = vec!["n"];
        let system = System::new(rows, vars);

        let expected = Err(Error::Contradiction);
        let actual = solve(system);

        assert_eq!(expected, actual);
    }
}
