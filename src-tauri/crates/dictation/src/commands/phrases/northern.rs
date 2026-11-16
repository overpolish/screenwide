// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The commands of Swedish, Danish, Norwegian and Finnish.

use super::super::Command::{AddWord, Mark, RemoveWord, ScratchThat};
use super::{Phrases, NEW_LINE, NEW_PARAGRAPH};

pub(super) const SWEDISH: Phrases = Phrases {
  said: &[
    ("stryk det", ScratchThat),
    ("ny rad", NEW_LINE),
    ("nytt stycke", NEW_PARAGRAPH),
    ("kommatecken", Mark(',')),
    ("komma", Mark(',')),
    ("punkt", Mark('.')),
    ("frågetecken", Mark('?')),
    ("utropstecken", Mark('!')),
    ("kolon", Mark(':')),
    ("semikolon", Mark(';')),
    ("lägg till ord", AddWord),
    ("ta bort ord", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "en|ett två tre fyra fem sex sju åtta nio tio",
  hashtag: None,
  quotes: None,
};

pub(super) const DANISH: Phrases = Phrases {
  said: &[
    ("slet det", ScratchThat),
    ("ny linje", NEW_LINE),
    ("nyt afsnit", NEW_PARAGRAPH),
    ("komma", Mark(',')),
    ("punktum", Mark('.')),
    ("spørgsmålstegn", Mark('?')),
    ("udråbstegn", Mark('!')),
    ("kolon", Mark(':')),
    ("semikolon", Mark(';')),
    ("tilføj ord", AddWord),
    ("fjern ord", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "en|et to tre fire fem seks syv otte ni ti",
  hashtag: None,
  quotes: None,
};

pub(super) const NORWEGIAN: Phrases = Phrases {
  said: &[
    ("slett det", ScratchThat),
    ("ny linje", NEW_LINE),
    ("nytt avsnitt", NEW_PARAGRAPH),
    ("komma", Mark(',')),
    ("punktum", Mark('.')),
    ("spørsmålstegn", Mark('?')),
    ("utropstegn", Mark('!')),
    ("kolon", Mark(':')),
    ("semikolon", Mark(';')),
    ("legg til ord", AddWord),
    ("fjern ord", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "en|ett|ei|ein|eitt to tre fire fem seks sju|syv åtte ni ti",
  hashtag: None,
  quotes: None,
};

pub(super) const FINNISH: Phrases = Phrases {
  said: &[
    ("poista se", ScratchThat),
    ("uusi rivi", NEW_LINE),
    ("uusi kappale", NEW_PARAGRAPH),
    ("pilkku", Mark(',')),
    ("piste", Mark('.')),
    ("kysymysmerkki", Mark('?')),
    ("huutomerkki", Mark('!')),
    ("kaksoispiste", Mark(':')),
    ("puolipiste", Mark(';')),
    ("lisää sana", AddWord),
    ("poista sana", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "yksi kaksi kolme neljä viisi kuusi seitsemän kahdeksan yhdeksän kymmenen",
  hashtag: None,
  quotes: None,
};
