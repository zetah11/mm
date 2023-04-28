use std::collections::HashMap;

use mm_eval::eval::Evaluator;
use mm_eval::melody::{Melody, Node};
use mm_eval::{Factor, Length, Name};
use rational::extras::r;

fn main() {
    let a = Melody {
        length: Length::one(),
        node: Node::Note('a'),
    };
    let b = Melody {
        length: Length::Bounded(r(1, 2)),
        node: Node::Pause,
    };
    let c = Melody {
        length: Length::Bounded(r(1, 2)),
        node: Node::Note('c'),
    };

    let seq = [a, b, c];
    let melody = Melody {
        length: Length::Bounded(r(2, 1)),
        node: Node::Sequence(&seq),
    };

    let scale = Melody {
        length: Length::Bounded(r(3, 2)),
        node: Node::Scale(Factor(r(3, 4)), &melody),
    };

    let deep = Melody {
        length: Length::Bounded(r(2, 1)),
        node: Node::Note('d'),
    };

    let stack = [scale, deep];
    let whole = Melody {
        length: Length::Bounded(r(2, 1)),
        node: Node::Stack(&stack),
    };

    let name = Name("main".into());
    let program = HashMap::from([(name.clone(), &whole)]);

    let eval = Evaluator::new(program, name);
    for (note, start, length) in eval.iter() {
        println!("{note:?} at {} for {:?}", start.0, length);
    }
}
