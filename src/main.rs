mod error;
mod file;

use std::io::{stderr, stdin};
use std::path::{Path, PathBuf};
use std::time::Duration;

use mm_eval::eval::Evaluator;
use mm_eval::Name;
use mm_media::midi::Pitch;
use mm_media::{midi, svg};
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use typed_arena::Arena;

const MAX_DEPTH: usize = 7;
const MAX_NOTES: usize = 1000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (args, paths) = Args::new(std::env::args().skip(1));

    if args.watch {
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |ev: DebounceEventResult| match ev {
                Ok(e) => compile(&args, e.into_iter().map(|e| e.path)).unwrap(),
                Err(e) => println!("watch error {e:?}"),
            },
        )?;

        for path in paths {
            debouncer
                .watcher()
                .watch(&path, RecursiveMode::NonRecursive)?;
        }

        stdin().read_line(&mut String::new())?;
        Ok(())
    } else {
        compile(&args, paths)
    }
}

fn compile(
    args: &Args,
    paths: impl IntoIterator<Item = PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let implicits = Arena::new();
    let explicits = Arena::new();

    let sources = file::get_sources(paths)?;
    let sources = sources.cache();

    for (path, source) in sources.iter() {
        let program = match mm_eval::compile(&implicits, &explicits, source) {
            Ok(program) => program,
            Err(es) => {
                let mut writer = stderr().lock();

                for e in es {
                    sources.report(&mut writer, e).unwrap();
                }

                return Ok(());
            }
        };

        let entry = Name("it");
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
    let notes = eval.iter().take(MAX_NOTES);

    match kind {
        Kind::Midi => midi::write(notes, &out)?,
        Kind::Svg => svg::write(notes, &out)?,
    }

    Ok(())
}

struct Args {
    make_midi: bool,
    make_svg: bool,
    watch: bool,
}

impl Args {
    pub fn new(args: impl IntoIterator<Item = String>) -> (Self, Vec<PathBuf>) {
        let mut make_midi = false;
        let mut make_svg = false;
        let mut watch = false;

        let mut paths = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-m" | "--midi" => make_midi = true,
                "-s" | "--svg" => make_svg = true,
                "-w" | "--watch" => watch = true,
                _ => paths.push(PathBuf::from(arg)),
            }
        }

        if !(make_midi || make_svg) {
            make_midi = true;
        }

        let args = Args {
            make_midi,
            make_svg,
            watch,
        };

        (args, paths)
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
