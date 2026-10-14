// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Ten pixels a character, whatever the size, in a picture whose lines take
/// 80 characters.
const FRAME: CaptionFrame = CaptionFrame {
  width: 1_000,
  height: 1_000,
};
const DURATION_MS: f64 = 100_000.0;

fn measure(text: &str, _face: CaptionFace, _size: f64) -> f64 {
  text.chars().count() as f64 * 10.0
}

/// Words said one after another, 300 ms each with 100 ms between, from 0.
fn said(text: &str) -> Vec<HeardWord> {
  text
    .split(' ')
    .enumerate()
    .map(|(index, word)| HeardWord {
      start_ms: index as f64 * 400.0,
      end_ms: index as f64 * 400.0 + 300.0,
      text: word.to_owned(),
    })
    .collect()
}

fn range(source_ms: [u64; 2], output_ms: u64) -> TimelineRange {
  TimelineRange {
    output_start_us: output_ms * 1_000,
    source_start_us: source_ms[0] * 1_000,
    source_end_us: source_ms[1] * 1_000,
    playback_rate: 1.0,
  }
}

struct Case {
  words: Vec<HeardWord>,
  corrections: Vec<Correction>,
  ranges: Vec<TimelineRange>,
  allowed: Vec<String>,
  captions: RecordingCaptions,
}

impl Case {
  fn new(text: &str) -> Self {
    Self {
      words: said(text),
      corrections: Vec::new(),
      ranges: vec![range([0, DURATION_MS as u64], 0)],
      allowed: Vec::new(),
      captions: RecordingCaptions {
        enabled: true,
        ..Default::default()
      },
    }
  }

  fn with(mut self, change: impl FnOnce(&mut CaptionSettings)) -> Self {
    change(&mut self.captions.settings);
    self
  }

  fn styled(mut self, change: impl FnOnce(&mut CaptionStyle)) -> Self {
    change(&mut self.captions.style);
    self
  }

  fn build(&self) -> Vec<Caption> {
    build(
      &CaptionSource {
        words: &self.words,
        language: Some("en"),
        corrections: &self.corrections,
        allowed_words: &self.allowed,
        ranges: &self.ranges,
        source_duration_ms: DURATION_MS,
      },
      &self.captions,
      FRAME,
      &mut measure,
    )
  }

  fn texts(&self) -> Vec<String> {
    self
      .build()
      .into_iter()
      .map(|caption| caption.text)
      .collect()
  }
}

#[test]
fn ends_each_sentence_and_shares_a_long_one_out_evenly() {
  let case = Case::new("one two three. four five six seven eight nine").styled(|style| {
    style.max_words = 4;
    style.lines = CaptionLines::One;
  });
  assert_eq!(
    case.texts(),
    ["one two three.", "four five six", "seven eight nine"]
  );
  let early = Case::new("Right. Two three four").styled(|style| style.max_words = 4);
  assert_eq!(early.texts(), ["Right.", "Two three four"]);
}

#[test]
fn sets_a_caption_too_wide_for_one_line_on_two_split_where_the_longer_is_shortest() {
  let words = [
    "a".repeat(30),
    "b".repeat(10),
    "c".repeat(10),
    "d".repeat(30),
    "e".repeat(20),
  ];
  let case = Case::new(&words.join(" "));
  assert_eq!(
    case.texts(),
    [format!(
      "{} {} {}\n{} {}",
      words[0], words[1], words[2], words[3], words[4]
    )]
  );
  let narrow = Case::new("aaaa bb cc dddd").styled(|style| style.max_words = 6);
  assert_eq!(narrow.texts(), ["aaaa bb cc dddd"]);
}

#[test]
fn wraps_by_width_before_the_word_limit() {
  // Fifty characters a word: one a line at 80 characters a line.
  let long = "x".repeat(50);
  let case = Case::new(&[long.as_str(); 3].join(" "));
  let captions = case.build();
  assert_eq!(captions.len(), 2);
  assert_eq!(captions[0].lines.len(), 2);
}

#[test]
fn holds_rolling_words_on_one_line_as_long_as_each_few_shown_fit() {
  // Thirty characters a word: two fit in 80, and only two show at once.
  let word = "x".repeat(30);
  let case = Case::new(&[word.as_str(); 6].join(" ")).styled(|style| {
    style.max_words = 6;
    style.visible_words = 2;
  });
  let captions = case.build();
  assert_eq!(captions.len(), 1);
  assert_eq!(captions[0].lines.len(), 1);
  assert_eq!(captions[0].lines[0].len(), 6);
}

#[test]
fn runs_across_a_cut_but_never_across_words_the_edit_moved() {
  // "two" (400-700) is cut: the caption flows from "one" to "three".
  let mut cut = Case::new("one two three four");
  cut.ranges = vec![range([0, 350], 0), range([750, 100_000], 350)];
  assert_eq!(cut.texts(), ["one three four"]);
  // "three four" (800-1500) plays first, then "one two": two captions.
  let mut moved = Case::new("one two three four");
  moved.ranges = vec![range([750, 100_000], 0), range([0, 750], 99_250)];
  assert_eq!(moved.texts(), ["three four", "one two"]);
}

#[test]
fn ends_a_caption_at_a_long_pause() {
  let mut case = Case::new("one two three four");
  for word in &mut case.words[2..] {
    word.start_ms += 1_500.0;
    word.end_ms += 1_500.0;
  }
  assert_eq!(case.texts(), ["one two", "three four"]);
}

#[test]
fn breaks_and_joins_where_placed_by_hand() {
  let mut split = Case::new("one two three four");
  // Between "two" (middle 550 ms) and "three" (middle 950 ms).
  split.captions.breaks = vec![CaptionBreak {
    source_at: 0.0075,
    kind: CaptionBreakKind::Split,
  }];
  assert_eq!(split.texts(), ["one two", "three four"]);

  let mut joined = Case::new("one two. three four").styled(|style| {
    style.max_words = 3;
    style.lines = CaptionLines::One;
  });
  assert_eq!(joined.texts(), ["one two.", "three four"]);
  joined.captions.breaks = vec![CaptionBreak {
    source_at: 0.0075,
    kind: CaptionBreakKind::Join,
  }];
  // The joined gap no longer ends the first caption, so it takes the most
  // words it can around it.
  assert_eq!(joined.texts(), ["one two. three", "four"]);
}

#[test]
fn hides_and_shows_words_chosen_by_hand() {
  let mut case = Case::new("so um this is it").with(|settings| settings.hide_fillers = true);
  assert_eq!(case.texts(), ["so this is it"]);
  // "um" is said from 400 to 700 ms, "this" from 800 to 1,100 ms.
  case.captions.word_choices = vec![
    CaptionWordChoice {
      source_at: 0.0055,
      kind: CaptionWordKind::Shown,
    },
    CaptionWordChoice {
      source_at: 0.0095,
      kind: CaptionWordKind::Hidden,
    },
  ];
  assert_eq!(case.texts(), ["so um is it"]);
}

#[test]
fn reads_each_word_as_the_settings_ask() {
  let case = Case::new("Don't, um, say damn well-known!")
    .styled(|style| style.all_caps = true)
    .with(|settings| {
      settings.remove_punctuation = true;
      settings.hide_fillers = true;
      settings.censor_profanity = true;
    });
  assert_eq!(case.texts(), ["DON'T SAY D*** WELL-KNOWN"]);
  let hidden = Case::new("say damn").with(|settings| {
    settings.censor_profanity = true;
    settings.profanity = CaptionProfanity::Hide;
  });
  assert_eq!(hidden.texts(), ["say ****"]);
  let mut allowed = Case::new("say damn").with(|settings| {
    settings.censor_profanity = true;
    settings.profanity = CaptionProfanity::Remove;
  });
  assert_eq!(allowed.texts(), ["say"]);
  allowed.allowed = vec!["damn".to_owned()];
  assert_eq!(allowed.texts(), ["say damn"]);
}

#[test]
fn shows_a_correction_sharing_its_stretch_among_its_words() {
  let mut case = Case::new("we gonna go").styled(|style| style.lines = CaptionLines::One);
  case.corrections = vec![Correction {
    start_ms: 400.0,
    end_ms: 700.0,
    text: "are going to".to_owned(),
  }];
  let captions = case.build();
  assert_eq!(captions[0].text, "we are going to go");
  let corrected: Vec<(f64, f64)> = captions[0].lines[0][1..4]
    .iter()
    .map(|word| (word.start_ms, word.end_ms))
    .collect();
  assert_eq!(corrected, [(400.0, 490.0), (490.0, 640.0), (640.0, 700.0)]);
}

#[test]
fn keeps_captions_up_through_the_speakers_usual_pauses_and_no_gaps_through_longer() {
  // Four 400 ms pauses teach a usual pause of 400 ms, so the speaker stops
  // at 800 ms: the 600 ms pause is a breath, the 1 s one a stop, and the
  // 4.5 s one silence even No gaps leaves.
  let mut case = Case::new("a b c d e f g h").styled(|style| style.max_words = 1);
  let starts = [
    0.0, 700.0, 1_400.0, 2_100.0, 2_800.0, 3_700.0, 5_000.0, 9_800.0,
  ];
  for (word, start) in case.words.iter_mut().zip(starts) {
    word.start_ms = start;
    word.end_ms = start + 300.0;
  }
  let ends =
    |case: &Case| -> Vec<f64> { case.build().iter().map(|caption| caption.end_ms).collect() };
  assert_eq!(
    ends(&case),
    [700.0, 1_400.0, 2_100.0, 2_800.0, 3_700.0, 4_400.0, 5_700.0, 10_500.0]
  );
  let bridged = case.with(|settings| settings.no_gaps = true);
  assert_eq!(
    ends(&bridged),
    [700.0, 1_400.0, 2_100.0, 2_800.0, 3_700.0, 5_000.0, 5_700.0, 10_500.0]
  );
}

#[test]
fn writes_subtitle_files() {
  // A caption too wide for one line over two; the 100 ms gap before "c" is
  // closed.
  let long = "a".repeat(78);
  let captions = Case::new(&format!("{long} <b> c"))
    .styled(|style| style.max_words = 2)
    .build();
  assert_eq!(
    srt(&captions),
    format!(
      "1\n00:00:00,000 --> 00:00:00,800\n{long}\n<b>\n\n2\n00:00:00,800 --> 00:00:01,500\nc\n\n"
    )
  );
  assert_eq!(
    vtt(&captions),
    format!("WEBVTT\n\n00:00:00.000 --> 00:00:00.800\n{long}\n&lt;b&gt;\n\n00:00:00.800 --> 00:00:01.500\nc\n\n")
  );
}

#[test]
fn refuses_breaks_out_of_order() {
  let captions = RecordingCaptions {
    breaks: vec![
      CaptionBreak {
        source_at: 0.5,
        kind: CaptionBreakKind::Split,
      },
      CaptionBreak {
        source_at: 0.2,
        kind: CaptionBreakKind::Join,
      },
    ],
    ..Default::default()
  };
  assert!(captions.validate().is_err());
}

#[test]
fn leaves_out_the_caption_holding_a_hidden_word_and_keeps_the_others_timed() {
  let mut case = Case::new("one two three. four five six seven eight nine").styled(|style| {
    style.max_words = 4;
    style.lines = CaptionLines::One;
  });
  let all = case.build();
  // "four" is said from 1.2 s, and starts the second caption.
  case.captions.hidden = vec![1_200];
  let kept = case.build();
  assert_eq!(kept, [all[0].clone(), all[2].clone()]);
  // Grouped otherwise, whichever caption now holds "four" goes instead.
  let regrouped = case.styled(|style| style.max_words = 2).texts();
  assert!(!regrouped.is_empty());
  assert!(regrouped.iter().all(|text| !text.contains("four")));
}

#[test]
fn keeps_the_caption_before_a_hidden_one_whose_last_word_ends_as_it_starts() {
  let mut case = Case::new("one two three. four five six").styled(|style| {
    style.max_words = 3;
    style.lines = CaptionLines::One;
  });
  // Said without a pause: "three." ends at 1.2 s, as "four" starts.
  for word in &mut case.words {
    word.end_ms = word.start_ms + 400.0;
  }
  case.captions.hidden = vec![1_200];
  assert_eq!(case.texts(), ["one two three."]);
}

/// Each caption's words set in the accent.
fn emphasized(captions: &[Caption]) -> Vec<Vec<&str>> {
  captions
    .iter()
    .map(|caption| {
      caption
        .lines
        .iter()
        .flatten()
        .filter(|word| word.emphasis)
        .map(|word| word.text.as_str())
        .collect()
    })
    .collect()
}

/// A choice on the `index`th word `said` lays out, at its middle.
fn emphasis_on(index: usize, kind: CaptionEmphasisKind) -> CaptionEmphasisChoice {
  CaptionEmphasisChoice {
    source_at: (index as f64 * 400.0 + 150.0) / DURATION_MS,
    kind,
  }
}

#[test]
fn sets_apart_each_captions_longest_word_unless_chosen_by_hand() {
  let mut case = Case::new("we built a remarkable thing. it works")
    .styled(|style| style.accent = CaptionAccent::Word);
  assert_eq!(
    emphasized(&case.build()),
    [vec!["remarkable"], vec!["works"]]
  );
  // A word chosen by hand takes the place of the one the rule picks, and a
  // word kept plain is passed over for the next longest.
  case.captions.emphasis_choices = vec![
    emphasis_on(1, CaptionEmphasisKind::Emphasized),
    emphasis_on(6, CaptionEmphasisKind::Plain),
  ];
  assert_eq!(emphasized(&case.build()), [vec!["built"], vec!["it"]]);
  // Without the accent on words, nothing is set apart.
  case.captions.style.accent = CaptionAccent::None;
  assert!(emphasized(&case.build()).iter().all(Vec::is_empty));
}

#[test]
fn stacks_a_word_or_two_a_line_with_the_word_set_apart_alone() {
  let case = Case::new("this is the secret to growth").styled(|style| {
    style.accent = CaptionAccent::Word;
    style.arrangement = CaptionArrangement::Stack;
    style.max_words = 6;
  });
  assert_eq!(case.texts(), ["this is\nthe\nsecret\nto growth"]);
}

#[test]
fn reads_captions_saved_with_values_no_longer_offered() {
  // Saved before revealing words was a switch, accents only key words, and
  // with templates since retired: the edit they are in still reads.
  let saved = serde_json::json!({
    "enabled": true,
    "style": { "template": "duet", "entrance": "words", "accent": "lines" },
  });
  let captions: RecordingCaptions = serde_json::from_value(saved).unwrap();
  assert_eq!(captions.style.template.as_str(), "clean");
  assert_eq!(captions.style.motion, CaptionMotion::default());
  assert_eq!(captions.style.accent, CaptionAccent::None);
}
