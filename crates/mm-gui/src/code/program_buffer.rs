use std::ops::Range;

use egui::TextBuffer;
use mm_eval::eval::Evaluator;
use mm_eval::note::Note;
use mm_eval::span::Span;
use mm_eval::{implicit, melody, Names};
use typed_arena::Arena;

pub struct ProgramBuffer<'a, N, Id> {
    name: Id,
    known_good: String,
    code: String,
    edits: Vec<isize>,
    dirty: bool,

    evaluator: Evaluator<'a, N, Id>,
}

impl<'a, N: Note, Id: Clone + Eq> ProgramBuffer<'a, N, Id> {
    pub fn new(name: Id, initial: &str, evaluator: Evaluator<'a, N, Id>) -> Self {
        Self {
            name,
            known_good: initial.into(),
            code: initial.into(),
            edits: vec![0; initial.len()],
            dirty: false,

            evaluator,
        }
    }

    pub fn name(&self) -> &Id {
        &self.name
    }

    pub fn evaluator(&self) -> &Evaluator<'a, N, Id> {
        &self.evaluator
    }

    pub fn update(
        &mut self,
        names: &mut Names,
        implicits: &'a Arena<implicit::Melody<'a, N, Id>>,
        explicits: &'a Arena<melody::Melody<'a, N, Id>>,
    ) {
        if self.dirty {
            match mm_eval::compile(names, implicits, explicits, self.name.clone(), &self.code) {
                Ok(program) => {
                    self.set_good(self.code.clone());

                    let entry = *program
                        .public
                        .first()
                        .expect("missing entry is reported by checking pass");
                    self.evaluator = Evaluator::new(program.defs, entry);
                }

                Err(errors) => {
                    println!("{} errors", errors.len());
                }
            }

            self.dirty = false;
        }
    }

    pub fn translate(&self, span: Span<Id>) -> Span<Id> {
        if span.source != self.name {
            return span;
        }

        let start = self.translate_pos(span.start);
        let end = self.translate_pos(span.end);
        Span::new(span.source, start..end)
    }

    fn set_good(&mut self, code: String) {
        self.edits = vec![0; code.len()];
        self.known_good = code;
    }
}

impl<N, Id> TextBuffer for ProgramBuffer<'_, N, Id> {
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_str(&self) -> &str {
        self.code.as_str()
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        self.do_edit(Edit::Insert { text, char_index });
        text.chars().count()
    }

    fn delete_char_range(&mut self, char_range: Range<usize>) {
        self.do_edit(Edit::Delete(char_range));
    }
}

enum Edit<'a> {
    Insert { text: &'a str, char_index: usize },

    Delete(Range<usize>),
}

impl<N, Id> ProgramBuffer<'_, N, Id> {
    fn do_edit(&mut self, edit: Edit) {
        match edit {
            Edit::Insert { text, char_index } => {
                let by = text.len() as isize;
                let pos = self.byte_index_from_char_index(char_index);
                self.code.insert_str(pos, text);

                let pos = self.inverse_pos(pos);

                for i in pos..self.edits.len() {
                    self.edits[i] += by;
                }
            }

            Edit::Delete(range) => {
                let start = self.byte_index_from_char_index(range.start);
                let end = self.byte_index_from_char_index(range.end);
                self.code.drain(start..end);

                let start = self.inverse_pos(start);
                let end = self.inverse_pos(end);

                let len = (end - start) as isize;

                for i in start..end {
                    self.edits[i] -= i as isize + 1;
                }

                for i in end..self.edits.len() {
                    self.edits[i] -= len;
                }
            }
        }

        self.dirty = true;
    }

    /// Translate from a position in the `self.code` string to a reasonable
    /// approximation in the `self.known_good` string.
    fn inverse_pos(&self, pos: usize) -> usize {
        let mut closest = None;
        for (i, off) in self.edits.iter().copied().enumerate() {
            let at = (i as isize + off) as usize;
            let dist = pos.abs_diff(at);

            if let Some((prev_dist, _)) = closest {
                if dist < prev_dist {
                    closest = Some((dist, i));
                }
            } else {
                closest = Some((dist, i));
            }
        }

        closest.unwrap_or((0, 0)).1
    }

    /// Translate a position in the `self.known_good` string to a reasonable
    /// approximation in the `self.code` string.
    fn translate_pos(&self, pos: usize) -> usize {
        (self.edits[pos] + pos as isize) as usize
    }
}
