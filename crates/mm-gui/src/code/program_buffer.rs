use std::ops::Range;

use egui::TextBuffer;
use mm_eval::span::Span;

struct ProgramBuffer<Id> {
    name: Id,
    known_good: String,
    code: String,
    edits: Vec<isize>,
}

impl<Id: Eq> ProgramBuffer<Id> {
    // ...

    pub fn translate(&self, span: Span<Id>) -> Span<Id> {
        if span.source != self.name {
            return span;
        }

        let start = self.translate_pos(span.start);
        let end = self.translate_pos(span.end);
        Span::new(span.source, start..end)
    }

    fn reset_edits(&mut self) {
        self.edits.clear();
    }

    fn translate_pos(&self, pos: usize) -> usize {
        (self.edits[pos] + pos as isize) as usize
    }
}

impl<Id> TextBuffer for ProgramBuffer<Id> {
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

impl<Id> ProgramBuffer<Id> {
    fn do_edit(&mut self, edit: Edit) {
        match edit {
            Edit::Insert { text, char_index } => {
                let by = text.len() as isize;
                let pos = self.byte_index_from_char_index(char_index);

                for i in pos..self.edits.len() {
                    self.edits[i] += by;
                }

                self.code.insert_str(pos, text);
            }

            Edit::Delete(range) => {
                let start = self.byte_index_from_char_index(range.start);
                let end = self.byte_index_from_char_index(range.end);

                let len = (end - start) as isize;

                for i in start..end {
                    self.edits[i] -= i as isize + 1;
                }

                for i in end..self.edits.len() {
                    self.edits[i] -= len;
                }

                self.code.drain(start..end);
            }
        }
    }
}
