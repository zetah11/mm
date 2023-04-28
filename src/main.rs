use mm_eval::eval::Evaluator;
use mm_eval::{CompilerState, Name};
use typed_arena::Arena;

fn main() {
    let implicits = Arena::new();
    let explicits = Arena::new();

    let state: CompilerState<char> = CompilerState::new(&implicits, &explicits);

    let source = r#"
        it = 1/2 (A, 1/2 (B, 1/2 C))
    "#;

    let program = state.compile(source).unwrap();
    let name = Name("it".into());

    let eval = Evaluator::new(program, name);
    for (note, start, length) in eval.iter() {
        println!("{note:?} at {} for {:?}", start.0, length);
    }
}
