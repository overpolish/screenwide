// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The notes as Markdown: a title, then what was said in the order it was
//! said, a paragraph for each turn, its speaker in bold before it. The file
//! is written again whole as each line comes, through a file beside it, so a
//! crash leaves the last whole notes rather than half a file. Text written
//! into the file is English, as text written into exported files is.

use std::fmt::Write as _;
use std::path::Path;

use screenwide_speech::runner::notes::{NoteLine, SpeakerTurn};

use super::echo::without_echo;
use super::speakers::{attributed, Who};

/// Notes being taken, or taken.
#[derive(Debug, Default)]
pub struct Document {
  title: String,
  lines: Vec<NoteLine>,
  turns: Option<Vec<SpeakerTurn>>,
}

impl Document {
  /// Notes headed `title`, such as when the call was.
  pub fn new(title: String) -> Self {
    Self {
      title,
      ..Self::default()
    }
  }

  /// Adds the lines just written.
  pub fn add(&mut self, lines: Vec<NoteLine>) {
    self.lines.extend(lines);
    self.lines.sort_by_key(|line| line.start_ms);
  }

  /// Adds the last lines, and tells the voices apart by `turns`.
  pub fn finish(&mut self, lines: Vec<NoteLine>, turns: Vec<SpeakerTurn>) {
    self.add(lines);
    self.turns = Some(turns);
  }

  pub fn markdown(&self) -> String {
    let lines = without_echo(&self.lines);
    let mut out = format!("# {}\n", self.title);
    let mut speaking = None;
    for (who, text) in attributed(&lines, self.turns.as_deref()).said {
      if speaking == Some(who) {
        out.push(' ');
      } else {
        // A blank line between turns, so each is its own paragraph.
        if speaking.is_some() {
          out.push('\n');
        }
        let _ = write!(out, "\n**{}:** ", label(who));
        speaking = Some(who);
      }
      out.push_str(text.trim());
    }
    out.push('\n');
    out
  }

  /// The speaker model's voice for each speaker number in the notes, from 1.
  pub fn voices(&self) -> Vec<u32> {
    attributed(&without_echo(&self.lines), self.turns.as_deref()).voices
  }

  /// Writes the notes to `path`, whole or not at all.
  pub fn save(&self, path: &Path) -> std::io::Result<()> {
    write_whole(path, &self.markdown())
  }
}

/// Heads the notes at `path` with `title` in place of the title they were
/// written under, as their project was renamed while they were taken.
pub fn retitle(path: &Path, title: &str) -> std::io::Result<()> {
  let markdown = std::fs::read_to_string(path)?;
  write_whole(path, &retitled(&markdown, title))
}

fn retitled(markdown: &str, title: &str) -> String {
  match markdown.strip_prefix("# ") {
    Some(headed) => {
      let body = headed.split_once('\n').map_or("", |(_, body)| body);
      format!("# {title}\n{body}")
    }
    None => format!("# {title}\n\n{markdown}"),
  }
}

/// Writes `markdown` to `path` through a file beside it, so a crash leaves
/// the last whole notes.
fn write_whole(path: &Path, markdown: &str) -> std::io::Result<()> {
  let writing = path.with_extension("md.part");
  std::fs::write(&writing, markdown)?;
  std::fs::rename(writing, path)
}

fn label(who: Who) -> String {
  match who {
    Who::Me => "Me".to_owned(),
    Who::Others => "Others".to_owned(),
    Who::Speaker(number) => format!("Speaker {number}"),
  }
}

#[cfg(test)]
mod tests {
  use screenwide_speech::runner::notes::{NoteLine, NoteSource, SpeakerTurn};

  use super::{retitled, Document};

  fn line(source: NoteSource, start_ms: u64, text: &str) -> NoteLine {
    NoteLine {
      source,
      start_ms,
      end_ms: start_ms + 1_500,
      text: text.to_owned(),
      words: Vec::new(),
    }
  }

  #[test]
  fn each_turn_is_a_paragraph_under_its_speaker() {
    let mut notes = Document::new("Notes".to_owned());
    notes.add(vec![
      line(NoteSource::Microphone, 3_000, "Friday works."),
      line(NoteSource::System, 0, "Can we ship Thursday?"),
      line(NoteSource::System, 1_600, "Or Friday?"),
    ]);
    assert_eq!(
      notes.markdown(),
      "# Notes\n\n**Others:** Can we ship Thursday? Or Friday?\n\n**Me:** Friday works.\n"
    );
    notes.finish(
      Vec::new(),
      vec![SpeakerTurn {
        start_ms: 0,
        end_ms: 3_000,
        speaker: 2,
      }],
    );
    assert!(notes
      .markdown()
      .contains("**Speaker 1:** Can we ship Thursday? Or Friday?"));
  }

  #[test]
  fn retitling_replaces_only_the_heading() {
    assert_eq!(
      retitled("# Notes 2026\n\n**Me:** # Not a heading\n", "Launch call"),
      "# Launch call\n\n**Me:** # Not a heading\n"
    );
    assert_eq!(
      retitled("**Me:** Hi\n", "Launch call"),
      "# Launch call\n\n**Me:** Hi\n"
    );
  }
}
