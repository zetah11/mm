mod error;
mod file;

use std::io::{stderr, stdin};
use std::path::{Path, PathBuf};
use std::time::Duration;

use error::SourceId;
use mm_eval::eval::Evaluator;
use mm_eval::{Arena, Names};
use mm_media::midi::Pitch;
use mm_media::{midi, svg};
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};

const MAX_DEPTH: usize = 20;
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
    let mut alloc = &Arena::new();

    let sources = file::get_sources(paths)?;
    let sources = sources.cache();

    let mut names = Names::new();

    for (id, path, source) in sources.iter() {
        let mut program = match mm_eval::compile(&mut alloc, &mut names, id, source) {
            Ok(program) => program,
            Err(es) => {
                let mut writer = stderr().lock();

                for e in es {
                    sources.report(&mut writer, &names, e).unwrap();
                }

                return Ok(());
            }
        };

        if program.public.len() != 1 {
            eprintln!("Multiple possible entrypoints. Skipping.");
            continue;
        }

        let entry = program.public.pop().unwrap();
        let eval = Evaluator::new(&program.defs, entry).with_max_depth(MAX_DEPTH);

        if args.make_midi {
            write(Kind::Midi, path, &eval)?;
        }

        if args.make_svg {
            write(Kind::Svg, path, &eval)?;
        }
    }

    Ok(())
}

fn write<'a>(
    kind: Kind,
    path: &Path,
    eval: &Evaluator<Pitch, SourceId, &'a Arena<'a, Pitch, SourceId>>,
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
