use std::collections::HashMap;

use mm_eval::eval::Evaluator;
use mm_eval::melody::{Factor, Length, Melody, Name, Node};
use rational::Rational;

fn main() {
    let a = Melody {
        length: Length(Rational::one()),
        node: Node::Note('a'),
    };
    let b = Melody {
        length: Length(Rational::new(1, 2)),
        node: Node::Pause,
    };
    let c = Melody {
        length: Length(Rational::new(1, 2)),
        node: Node::Note('c'),
    };

    let seq = [a, b, c];
    let melody = Melody {
        length: Length(Rational::new(2, 1)),
        node: Node::Sequence(&seq),
    };

    let scale = Melody {
        length: Length(Rational::new(3, 2)),
        node: Node::Scale(Factor(Rational::new(3, 4)), &melody),
    };

    let deep = Melody {
        length: Length(Rational::new(2, 1)),
        node: Node::Note('d'),
    };

    let stack = [scale, deep];
    let whole = Melody {
        length: Length(Rational::new(2, 1)),
        node: Node::Stack(&stack),
    };

    let name = Name("main".into());
    let program = HashMap::from([(name.clone(), &whole)]);

    let eval = Evaluator::new(program, name);
    for (note, start, length) in eval.iter() {
        println!("{note:?} at {} for {}", start.0, length.0);
    }
}
