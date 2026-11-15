// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The commands of French, German, Spanish, Italian, Portuguese and Dutch.

use super::super::Command::{AddWord, Mark, RemoveWord, ScratchThat};
use super::{Phrases, NEW_LINE, NEW_PARAGRAPH};

pub(super) const FRENCH: Phrases = Phrases {
  said: &[
    ("efface ça", ScratchThat),
    ("à la ligne", NEW_LINE),
    ("nouvelle ligne", NEW_LINE),
    ("nouveau paragraphe", NEW_PARAGRAPH),
    ("virgule", Mark(',')),
    ("point", Mark('.')),
    ("point final", Mark('.')),
    ("point d'interrogation", Mark('?')),
    ("point d'exclamation", Mark('!')),
    ("ajouter le mot", AddWord),
    ("ajoute le mot", AddWord),
    ("ajoutez le mot", AddWord),
    ("supprimer le mot", RemoveWord),
    ("supprime le mot", RemoveWord),
    ("supprimez le mot", RemoveWord),
  ],
  spaced: true,
};

pub(super) const GERMAN: Phrases = Phrases {
  said: &[
    ("streich das", ScratchThat),
    ("lösch das", ScratchThat),
    ("neue Zeile", NEW_LINE),
    ("neuer Absatz", NEW_PARAGRAPH),
    ("Komma", Mark(',')),
    ("Punkt", Mark('.')),
    ("Fragezeichen", Mark('?')),
    ("Ausrufezeichen", Mark('!')),
    ("Doppelpunkt", Mark(':')),
    ("Semikolon", Mark(';')),
    ("Wort hinzufügen", AddWord),
    ("Wort entfernen", RemoveWord),
  ],
  spaced: true,
};

pub(super) const SPANISH: Phrases = Phrases {
  said: &[
    ("borra eso", ScratchThat),
    ("nueva línea", NEW_LINE),
    ("nuevo párrafo", NEW_PARAGRAPH),
    ("coma", Mark(',')),
    ("punto", Mark('.')),
    ("punto final", Mark('.')),
    ("signo de interrogación", Mark('?')),
    ("signo de exclamación", Mark('!')),
    ("dos puntos", Mark(':')),
    ("punto y coma", Mark(';')),
    ("añadir palabra", AddWord),
    ("agregar palabra", AddWord),
    ("quitar palabra", RemoveWord),
    ("eliminar palabra", RemoveWord),
  ],
  spaced: true,
};

pub(super) const ITALIAN: Phrases = Phrases {
  said: &[
    ("cancella questo", ScratchThat),
    ("a capo", NEW_LINE),
    ("nuova riga", NEW_LINE),
    ("nuovo paragrafo", NEW_PARAGRAPH),
    ("virgola", Mark(',')),
    ("punto", Mark('.')),
    ("punto interrogativo", Mark('?')),
    ("punto esclamativo", Mark('!')),
    ("due punti", Mark(':')),
    ("punto e virgola", Mark(';')),
    ("aggiungi parola", AddWord),
    ("rimuovi parola", RemoveWord),
  ],
  spaced: true,
};

pub(super) const PORTUGUESE: Phrases = Phrases {
  said: &[
    ("apaga isso", ScratchThat),
    ("nova linha", NEW_LINE),
    ("novo parágrafo", NEW_PARAGRAPH),
    ("vírgula", Mark(',')),
    ("ponto", Mark('.')),
    ("ponto final", Mark('.')),
    ("ponto de interrogação", Mark('?')),
    ("ponto de exclamação", Mark('!')),
    ("dois pontos", Mark(':')),
    ("ponto e vírgula", Mark(';')),
    ("adicionar palavra", AddWord),
    ("remover palavra", RemoveWord),
  ],
  spaced: true,
};

pub(super) const DUTCH: Phrases = Phrases {
  said: &[
    ("schrap dat", ScratchThat),
    ("nieuwe regel", NEW_LINE),
    ("nieuwe alinea", NEW_PARAGRAPH),
    ("komma", Mark(',')),
    ("punt", Mark('.')),
    ("vraagteken", Mark('?')),
    ("uitroepteken", Mark('!')),
    ("dubbele punt", Mark(':')),
    ("puntkomma", Mark(';')),
    ("woord toevoegen", AddWord),
    ("woord verwijderen", RemoveWord),
  ],
  spaced: true,
};
