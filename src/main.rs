use mm_eval::eval::Evaluator;
use mm_eval::{CompilerState, Name};
use mm_media::midi::{self, Pitch};
use typed_arena::Arena;

const MAX_DEPTH: usize = 20;
const MAX_NOTES: usize = 1000;

const SOURCE: &str = r#"
    it = x, F
    x = A, G, 1/3 y
    y = D, E, 2 x
"#;

fn main() {
    let implicits = Arena::new();
    let explicits = Arena::new();

    let state: CompilerState<Pitch> = CompilerState::new(&implicits, &explicits);

    let program = state.compile(SOURCE).unwrap();
    let name = Name("it".into());

    let eval = Evaluator::new(program, name).with_max_depth(MAX_DEPTH);
    midi::write(eval.iter().take(MAX_NOTES), "out.mid").unwrap();
}
