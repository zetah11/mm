use mm_eval::eval::Evaluator;
use mm_eval::{compile, Name};
use mm_media::midi;
use typed_arena::Arena;

const MAX_DEPTH: usize = 100;
const MAX_NOTES: usize = 1000;

const SOURCE: &str = r#"
    it = x, F
    x = A, G, 1/3 y
    y = D, E, 2 x
"#;

fn main() {
    let implicits = Arena::new();
    let explicits = Arena::new();

    let program = compile(&implicits, &explicits, SOURCE).unwrap();
    let name = Name("it".into());

    let eval = Evaluator::new(program.defs, name).with_max_depth(MAX_DEPTH);
    midi::write(eval.iter().take(MAX_NOTES), "out.mid").unwrap();
}
