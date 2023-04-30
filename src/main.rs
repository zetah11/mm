mod error;
mod file;

use std::io::stderr;
use std::path::Path;

use mm_eval::eval::Evaluator;
use mm_eval::{compile, Name};
use mm_media::midi::Pitch;
use mm_media::{midi, svg};
use typed_arena::Arena;

const MAX_DEPTH: usize = 100;
const MAX_NOTES: usize = 1000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let implicits = Arena::new();
    let explicits = Arena::new();

    let args = Args::new(std::env::args().skip(1));

    let sources = file::get_sources(args.paths)?;
    let sources = sources.cache();

    for (path, source) in sources.iter() {
        let program = match compile(&implicits, &explicits, source) {
            Ok(program) => program,
            Err(es) => {
                let mut writer = stderr().lock();

                for e in es {
                    sources.report(&mut writer, e).unwrap();
                }

                return Ok(());
            }
        };

        let entry = Name("it".into());
        let eval = Evaluator::new(program.defs, entry).with_max_depth(MAX_DEPTH);
        let path = Path::new(path);

        if args.make_midi {
            write(Kind::Midi, path, &eval)?;
        }

        if args.make_svg {
            write(Kind::Svg, path, &eval)?;
        }
    }

    Ok(())
}

fn write(
    kind: Kind,
    path: &Path,
    eval: &Evaluator<Pitch>,
) -> Result<(), Box<dyn std::error::Error>> {
    let out = path.with_extension(kind.extension());

    match kind {
        Kind::Midi => midi::write(eval.iter().take(MAX_NOTES), &out)?,
        Kind::Svg => svg::write(eval.iter().take(MAX_NOTES), &out)?,
    }

    Ok(())
}

struct Args {
    make_midi: bool,
    make_svg: bool,
    paths: Vec<String>,
}

impl Args {
    pub fn new(args: impl IntoIterator<Item = String>) -> Self {
        let mut make_midi = false;
        let mut make_svg = false;

        let mut paths = Vec::new();

        for arg in args {
            match arg.as_str() {
                "--midi" => make_midi = true,
                "--svg" => make_svg = true,
                _ => paths.push(arg),
            }
        }

        if !(make_midi || make_svg) {
            make_midi = true;
        }

        Args {
            make_midi,
            make_svg,
            paths,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Kind {
    #[default]
    Midi,
    Svg,
}

impl Kind {
    fn extension(&self) -> &'static str {
        match self {
            Self::Midi => "mid",
            Self::Svg => "svg",
        }
    }
}
