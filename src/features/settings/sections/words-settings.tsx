// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { X } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useState } from "react";

import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { TextField } from "../../../primitives/input-fields/text-field";
import { glide, leave, reveal } from "../../../primitives/island/island-motion";
import { SettingsGroup } from "../settings-group";
import { clearIn } from "../settings-motion";

import type { TranscriptionControls } from "./use-transcription";

// The word list: names and terms dictation, recording transcripts and voice
// notes write as they are spelled here. Each word is a field to edit in
// place, kept as it leaves the field or on Return, and taken off emptied or
// with its remove; a field after them adds one. Words added by voice while
// dictating arrive here too.

function WordRow({
  onChange,
  word,
}: {
  onChange: (word: string) => void;
  word: string;
}) {
  const [draft, setDraft] = useState(word);
  const [shown, setShown] = useState(word);
  if (word !== shown) {
    setShown(word);
    setDraft(word);
  }
  const keep = () => {
    if (draft.trim() !== word) onChange(draft.trim());
  };
  return (
    <div className="flex items-center gap-separate">
      <TextField
        aria-label={t("settings-words-edit", { word })}
        className="grow"
        onBlur={keep}
        onChange={setDraft}
        onKeyDown={(event) => {
          if (event.key === "Enter") keep();
          if (event.key === "Escape") setDraft(word);
        }}
        value={draft}
      />
      <IconButton
        aria-label={t("settings-words-remove", { word })}
        className="text-fg-secondary enabled:hover:text-fg"
        onPress={() => {
          onChange("");
        }}
        variant="ghost"
      >
        <X aria-hidden className="size-icon" />
      </IconButton>
    </div>
  );
}

type Row = { key: number; word: string };

const NO_WORDS: string[] = [];
const still = { duration: 0 };

// Keys that follow a word through an edit, so an edited row stays put
// rather than leaving and arriving again: a word still listed keeps its
// key, and where the count is unchanged a row whose word went takes the
// changed word in its place. Only a word added or removed comes or goes.
function rekey(rows: Row[], words: string[]): Row[] {
  let next = Math.max(0, ...rows.map(({ key }) => key + 1));
  const kept = new Set<number>();
  const listed = new Set(words);
  return words.map((word, index) => {
    const same = rows.find((row) => row.word === word && !kept.has(row.key));
    const edited = rows.length === words.length ? rows[index] : undefined;
    const key =
      same?.key ??
      (edited && !listed.has(edited.word) && !kept.has(edited.key)
        ? edited.key
        : next++);
    kept.add(key);
    return { key, word };
  });
}

export function WordsSettings({
  controls,
}: {
  controls: TranscriptionControls;
}) {
  const reduce = useReducedMotion() ?? false;
  const [draft, setDraft] = useState("");
  const words = controls.state?.words ?? NO_WORDS;
  const [rows, setRows] = useState(() => rekey([], words));
  const [seen, setSeen] = useState(words);
  if (words !== seen) {
    setSeen(words);
    setRows(rekey(rows, words));
  }
  const add = () => {
    const word = draft.trim();
    if (!word) return;
    controls.setWords([...words, word]);
    setDraft("");
  };
  return (
    <SettingsGroup
      description={t("settings-words-description")}
      footnote={t("settings-words-by-voice")}
      title={t("settings-words")}
    >
      <div className="flex flex-col">
        <AnimatePresence initial={false}>
          {rows.map(({ key, word }, index) => (
            <motion.div
              animate={{
                ...clearIn.animate,
                height: "auto",
                transitionEnd: { filter: "none", overflow: "visible" },
              }}
              exit={{
                ...clearIn.exit,
                height: 0,
                overflow: "hidden",
                transition: reduce
                  ? still
                  : { filter: leave, height: glide, opacity: leave },
              }}
              initial={{ ...clearIn.initial, height: 0, overflow: "hidden" }}
              key={key}
              transition={
                reduce ? still : { ...reveal.transition, height: glide }
              }
            >
              <div className="pb-separate">
                <WordRow
                  onChange={(next) => {
                    controls.setWords(
                      words
                        .map((listed, at) => (at === index ? next : listed))
                        .filter(Boolean),
                    );
                  }}
                  word={word}
                />
              </div>
            </motion.div>
          ))}
        </AnimatePresence>
        <div className="flex items-center gap-separate">
          <TextField
            aria-label={t("settings-words-new")}
            className="grow"
            onChange={setDraft}
            onKeyDown={(event) => {
              if (event.key === "Enter") add();
            }}
            placeholder={t("settings-words-new")}
            value={draft}
          />
          <Button isDisabled={!draft.trim()} onPress={add}>
            {t("settings-words-add")}
          </Button>
        </div>
      </div>
    </SettingsGroup>
  );
}
