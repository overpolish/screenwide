// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A box being typed into whose look sets chosen words apart. Its words are
//! drawn in their own faces and sizes, so neither platform's own text view
//! can say where a press lands or where a line up or down goes: both ask
//! here, where the words are laid out as the compositor draws them. The
//! chosen words follow each change, as the edit session's do.

use super::look;
use screenwide_text::captions::{ItemTyping, TextItemLayout};
use screenwide_text::emphasis::follow_edit;
use screenwide_text::{TextSpan, TextStyle};

pub struct TypedWords {
  look: Box<TextStyle>,
  text: String,
  emphasis: Vec<TextSpan>,
  /// Whether its words are laid out here however they are set: a caption's
  /// lines follow its style's arrangement, which no text view sets.
  always: bool,
}

impl TypedWords {
  /// The typing of `text` in `look` with the words of `emphasis` chosen,
  /// where the look sets any of them apart.
  pub fn new(text: &str, look: &TextStyle, emphasis: &[TextSpan]) -> Option<Self> {
    look::sets_words_apart(look, text, emphasis).then(|| Self {
      look: Box::new(look.clone()),
      text: text.to_owned(),
      emphasis: emphasis.to_vec(),
      always: false,
    })
  }

  /// The typing of a caption's words, `text` in `look`, the caption's
  /// style as it is typed into, with its key words `emphasis`.
  pub fn caption(text: &str, look: &TextStyle, emphasis: &[TextSpan]) -> Self {
    Self {
      look: Box::new(look.clone()),
      text: text.to_owned(),
      emphasis: emphasis.to_vec(),
      always: true,
    }
  }

  /// Carries the chosen words through the change that left `text`.
  pub fn follow(&mut self, text: &str) {
    if text != self.text {
      self.emphasis = follow_edit(&self.emphasis, &self.text, text);
      self.text = text.to_owned();
    }
  }

  /// Whether its words are laid out here: a caption's always, a text box's
  /// while any chosen word is left to set apart. Once none is, the box is
  /// set in one face and its platform's own text view reads it again.
  pub fn sets_apart(&self) -> bool {
    self.always || look::sets_words_apart(&self.look, &self.text, &self.emphasis)
  }

  /// `work` done with the text laid out at type size `font`, its lines
  /// lined up by the record's alignment `align`, every point from the top
  /// left of its block in the units `font` is in.
  pub fn with_lines<R>(&self, font: f64, align: u32, work: impl FnOnce(&mut ItemTyping) -> R) -> R {
    let mut measure = look::measure(&self.look);
    let share = f64::from(align.min(2)) / 2.0;
    let layout = TextItemLayout::new(
      &self.text,
      &self.look,
      &self.emphasis,
      font,
      share,
      &mut measure,
    );
    work(&mut ItemTyping {
      layout: &layout,
      measure: &mut measure,
    })
  }

  /// The offset of the text nearest `point`.
  pub fn offset_at(&self, font: f64, align: u32, point: (f64, f64)) -> usize {
    self.with_lines(font, align, |lines| {
      lines.layout.offset_at(&self.text, point, lines.measure)
    })
  }

  /// Where a line up or down from `offset` goes, keeping to `goal` across
  /// the block where a run of line moves has one: the offset, and the goal
  /// the next move keeps to. Past the first or last line it goes to the
  /// text's start or end.
  pub fn line_from(
    &self,
    (font, align): (f64, u32),
    offset: usize,
    down: bool,
    goal: Option<f64>,
  ) -> (usize, f64) {
    let text = &self.text;
    let offset = offset.min(text.len());
    self.with_lines(font, align, |lines| {
      let goal = goal.unwrap_or_else(|| lines.layout.point_of(text, offset, lines.measure).0);
      let line = text[..offset].matches('\n').count();
      let moved = if !down && line == 0 {
        0
      } else if down && line + 1 >= lines.layout.line_count() {
        text.len()
      } else {
        let to = if down { line + 1 } else { line - 1 };
        lines.layout.offset_in_line(text, to, goal, lines.measure)
      };
      (moved, goal)
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_text::TextAccent;

  fn editorial() -> TextStyle {
    TextStyle {
      accent: TextAccent::Word,
      ..TextStyle::default()
    }
  }

  #[test]
  fn types_only_a_box_that_sets_words_apart() {
    let cat = [TextSpan { start: 4, end: 7 }];
    assert!(TypedWords::new("big cat", &editorial(), &cat).is_some());
    assert!(TypedWords::new("big cat", &editorial(), &[]).is_none());
    assert!(TypedWords::new("big cat", &TextStyle::default(), &cat).is_none());
  }

  #[test]
  fn follows_the_typing_until_no_chosen_word_is_left() {
    let mut typed =
      TypedWords::new("big cat", &editorial(), &[TextSpan { start: 4, end: 7 }]).unwrap();
    typed.follow("a big cat");
    assert!(typed.sets_apart());
    typed.follow("a big ");
    assert!(!typed.sets_apart());
  }

  #[test]
  fn moves_a_line_down_and_back_to_where_it_began() {
    let typed = TypedWords::new(
      "big cat\nsat",
      &editorial(),
      &[TextSpan { start: 4, end: 7 }],
    )
    .unwrap();
    let (down, goal) = typed.line_from((20.0, 0), 2, true, None);
    assert!((8..=11).contains(&down));
    let (up, _) = typed.line_from((20.0, 0), down, false, Some(goal));
    assert_eq!(up, 2);
    // Past the last line it goes to the end.
    assert_eq!(typed.line_from((20.0, 0), down, true, Some(goal)).0, 11);
  }
}
