// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The commands of Polish, Czech, Russian, Ukrainian and Turkish.

use super::super::Command::{AddWord, Mark, RemoveWord, ScratchThat};
use super::{Phrases, END_LIST, NEW_LINE, NEW_PARAGRAPH};

pub(super) const POLISH: Phrases = Phrases {
  said: &[
    ("usuń to", ScratchThat),
    ("nowa linia", NEW_LINE),
    ("nowy akapit", NEW_PARAGRAPH),
    ("koniec listy", END_LIST),
    ("przecinek", Mark(',')),
    ("kropka", Mark('.')),
    ("znak zapytania", Mark('?')),
    ("wykrzyknik", Mark('!')),
    ("dwukropek", Mark(':')),
    ("średnik", Mark(';')),
    ("dodaj słowo", AddWord),
    ("usuń słowo", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "jeden|jedna|jedno dwa|dwie trzy cztery pięć sześć siedem osiem dziewięć dziesięć",
  hashtag: None,
  quotes: None,
};

pub(super) const CZECH: Phrases = Phrases {
  said: &[
    ("smaž to", ScratchThat),
    ("nový řádek", NEW_LINE),
    ("nový odstavec", NEW_PARAGRAPH),
    ("konec seznamu", END_LIST),
    ("čárka", Mark(',')),
    ("tečka", Mark('.')),
    ("otazník", Mark('?')),
    ("vykřičník", Mark('!')),
    ("dvojtečka", Mark(':')),
    ("středník", Mark(';')),
    ("přidat slovo", AddWord),
    ("odebrat slovo", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "jeden|jedna|jedno dva|dvě tři čtyři pět šest sedm osm devět deset",
  hashtag: None,
  quotes: None,
};

pub(super) const RUSSIAN: Phrases = Phrases {
  said: &[
    ("удали это", ScratchThat),
    ("новая строка", NEW_LINE),
    ("новый абзац", NEW_PARAGRAPH),
    ("конец списка", END_LIST),
    ("запятая", Mark(',')),
    ("точка", Mark('.')),
    ("вопросительный знак", Mark('?')),
    ("восклицательный знак", Mark('!')),
    ("двоеточие", Mark(':')),
    ("точка с запятой", Mark(';')),
    ("добавить слово", AddWord),
    ("удалить слово", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "один|одна|одно два|две три четыре пять шесть семь восемь девять десять",
  hashtag: None,
  quotes: None,
};

pub(super) const UKRAINIAN: Phrases = Phrases {
  said: &[
    ("видали це", ScratchThat),
    ("новий рядок", NEW_LINE),
    ("новий абзац", NEW_PARAGRAPH),
    ("кінець списку", END_LIST),
    ("кома", Mark(',')),
    ("крапка", Mark('.')),
    ("знак питання", Mark('?')),
    ("знак оклику", Mark('!')),
    ("двокрапка", Mark(':')),
    ("крапка з комою", Mark(';')),
    ("додати слово", AddWord),
    ("видалити слово", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "один|одна|одне два|дві три чотири пять шість сім вісім девять десять",
  hashtag: None,
  quotes: None,
};

pub(super) const TURKISH: Phrases = Phrases {
  said: &[
    ("bunu sil", ScratchThat),
    ("yeni satır", NEW_LINE),
    ("yeni paragraf", NEW_PARAGRAPH),
    ("liste sonu", END_LIST),
    ("virgül", Mark(',')),
    ("nokta", Mark('.')),
    ("soru işareti", Mark('?')),
    ("ünlem işareti", Mark('!')),
    ("iki nokta", Mark(':')),
    ("noktalı virgül", Mark(';')),
    ("kelime ekle", AddWord),
    ("kelime sil", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "bir iki üç dört beş altı yedi sekiz dokuz on",
  hashtag: None,
  quotes: None,
};
