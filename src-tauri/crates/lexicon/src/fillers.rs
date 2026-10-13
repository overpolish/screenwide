// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The hesitation sounds each language fills a pause with, and the words a
//! speaker starts over on.

use crate::fold::fold_filler_text;
use crate::{Finding, Unit};
use std::collections::HashSet;

/// The sounds by the language code the transcriber reports. Only sounds made
/// while thinking are listed: a word that is also said for its meaning
/// ("like", "well", "este") would be found in every sentence. English is read
/// for every language too, since "um" and "uh" turn up in all of them.
/// Spellings are as a transcript writes them; a sound drawn out ("ummm") is
/// matched by its shorter form.
const FILLER_SOUNDS: &[(&str, &[&str])] = &[
  ("ar", &["اه", "ام", "امم", "اممم", "ااه"]),
  ("cs", &["ehm", "eee", "hm", "hmm", "ee", "em"]),
  ("da", &["øh", "øhm", "æh", "hm", "hmm", "eh"]),
  ("de", &["äh", "ähm", "öh", "öhm", "hm", "hmm", "eh", "ehm"]),
  ("el", &["εε", "εμ", "εεμ", "χμ", "ε"]),
  (
    "en",
    &[
      "ah", "eh", "er", "erm", "hm", "hmm", "mhm", "mm", "uh", "uhm", "um",
    ],
  ),
  ("es", &["eh", "em", "ehm", "mm", "mmm"]),
  ("fa", &["اِم", "ام", "اه", "اوم"]),
  ("fi", &["öö", "ää", "hmm", "mm", "tota", "tuota", "niinku"]),
  ("fr", &["euh", "heu", "hum", "hm", "bah", "euhm"]),
  ("he", &["אה", "אמ", "אמם", "אהה", "הממ"]),
  ("hi", &["अं", "अम", "हम्म", "उम", "आं"]),
  ("hu", &["öö", "ööö", "ö", "hm", "hmm", "izé"]),
  ("id", &["eh", "em", "hmm", "anu", "eee"]),
  ("it", &["eh", "ehm", "em", "mm", "mmm", "uhm"]),
  (
    "ja",
    &[
      "えー",
      "えーと",
      "えっと",
      "あのー",
      "うーん",
      "んー",
      "ええと",
      "あー",
    ],
  ),
  ("ko", &["음", "어", "아", "으음", "어어"]),
  ("ms", &["eh", "em", "hmm", "anu"]),
  ("nl", &["eh", "uh", "ehm", "uhm", "hm", "hmm"]),
  ("no", &["eh", "øh", "æh", "ehm", "hm", "hmm"]),
  ("pl", &["yyy", "yy", "eee", "ee", "hmm", "em", "ehm"]),
  ("pt", &["é", "éé", "hã", "ãh", "ahn", "hum", "hmm", "eh"]),
  ("ro", &["ăă", "ăăă", "ă", "hm", "hmm", "ăm"]),
  ("ru", &["э", "ээ", "эм", "эээ", "мм", "ммм", "хм"]),
  ("sk", &["ehm", "eee", "ee", "hm", "hmm"]),
  ("sv", &["eh", "öh", "äh", "öhm", "hm", "hmm", "mm"]),
  ("th", &["เอ่อ", "อืม", "เออ", "อ่า"]),
  ("tr", &["ıı", "ııı", "ee", "eee", "hmm", "hım"]),
  ("uk", &["е", "ее", "еее", "ем", "мм", "ммм", "хм"]),
  ("vi", &["ờ", "ừm", "ừ", "à", "ơ", "ờm"]),
  ("zh", &["嗯", "呃", "额", "唔", "啊"]),
];

/// A word said again right after itself within this long still reads as the
/// speaker starting over, not two words.
const REPEAT_WITHIN_MS: f64 = 1_000.0;

/// The folded hesitation sounds to look for in a transcript in `language`,
/// English's always among them. Nynorsk shares Norwegian's list, and
/// Cantonese the Chinese written for it.
fn sounds_for(language: Option<&str>) -> HashSet<String> {
  let code = language.map(|code| match code {
    "nn" => "no",
    "yue" => "zh",
    other => other,
  });
  FILLER_SOUNDS
    .iter()
    .filter(|(list, _)| *list == "en" || Some(*list) == code)
    .flat_map(|(_, sounds)| sounds.iter())
    .map(|sound| fold_filler_text(sound))
    .collect()
}

/// Every filler `units` still keep: a hesitation sound in English or
/// `language`, and the first of a word said twice within a second, as a
/// speaker starting over does. Words are read as they show, so a word
/// corrected into a filler is one, and one corrected out of it is not. A word
/// already cut is never a filler. Fillers are in transcript order.
pub fn find_fillers(units: &[Unit], language: Option<&str>) -> Vec<Finding> {
  let sounds = sounds_for(language);
  let kept: Vec<(usize, &Unit, String)> = units
    .iter()
    .enumerate()
    .filter(|(_, unit)| !unit.removed)
    .map(|(index, unit)| (index, unit, fold_filler_text(unit.text)))
    .collect();
  let mut fillers = Vec::new();
  for (position, (index, unit, folded)) in kept.iter().enumerate() {
    if folded.is_empty() {
      continue;
    }
    let is_repeat = kept
      .get(position + 1)
      .is_some_and(|(_, next, next_folded)| {
        next_folded == folded && next.start_ms - unit.end_ms < REPEAT_WITHIN_MS
      });
    if sounds.contains(folded) || is_repeat {
      fillers.push(Finding {
        folded: folded.clone(),
        first: *index,
        last: *index,
      });
    }
  }
  fillers
}
