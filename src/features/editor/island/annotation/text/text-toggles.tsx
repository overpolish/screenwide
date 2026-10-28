// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { t } from "../../../../../i18n/i18n";
import { useGlyphMotion } from "../../../../annotations/options/glyph-motion";
import { OptionToggle } from "../../../../annotations/options/option-toggle";

/** A capital A as one shape, its counter cut out. */
const LETTER =
  "M10.2 4h3.6l6.2 16h-3.9l-1.3-3.6H9.2L7.9 20H4zM10.3 13.3h3.4L12 8.4z";

/** Without an outline, the letter alone; turned on, an outline is drawn
 * round it, starting at its foot, and turned off it is taken back. */
function OutlineGlyph({ outlined }: { outlined: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg aria-hidden className="size-icon" viewBox="0 0 24 24">
      <motion.path
        animate={
          outlined
            ? { opacity: 0.5, pathLength: 1 }
            : { opacity: 0, pathLength: 0 }
        }
        d={LETTER}
        fill="none"
        initial={false}
        stroke="currentColor"
        // Drawn in as a dash, whose ends meet at the letter's top corner
        // with caps rather than a join; round caps close it up there as the
        // round joins do everywhere else.
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={3.5}
        transition={motionOf.draw}
      />
      <path d={LETTER} fill="currentColor" fillRule="evenodd" />
    </svg>
  );
}

/** Each word of two lines of text, drawn as the alignment icons draw a
 * line, so it reads beside them: where it runs while its line comes on
 * whole, end to end with its neighbours, how far it moves to stand apart,
 * and how strong it is caught part way through coming on word by word, the
 * first strongest. */
const WORDS = [
  { apart: -1.5, from: 4, stepped: 1, to: 8, y: 8.5 },
  { apart: 0, from: 10.5, stepped: 0.8, to: 13.5, y: 8.5 },
  { apart: 1.5, from: 16, stepped: 0.6, to: 20, y: 8.5 },
  { apart: -1.5, from: 4, stepped: 0.45, to: 9, y: 15.5 },
  { apart: 1.5, from: 11.5, stepped: 0.3, to: 14, y: 15.5 },
];

/** Two lines of text. Coming on together, each line is unbroken; turned
 * on, the lines part into their words, which fade from the first to the
 * last, as words that come on one after another read caught part way;
 * turned off, they close up again. */
function WordsGlyph({ oneByOne }: { oneByOne: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeWidth={2.5}
      viewBox="0 0 24 24"
    >
      {WORDS.map(({ apart, from, stepped, to, y }, index) => (
        <motion.path
          animate={
            oneByOne ? { opacity: stepped, x: apart } : { opacity: 1, x: 0 }
          }
          d={`M${String(from)} ${String(y)}H${String(to)}`}
          initial={false}
          key={`${String(from)} ${String(y)}`}
          transition={{
            ...motionOf.move,
            delay: oneByOne && motionOf.draw.duration !== 0 ? index * 0.05 : 0,
          }}
        />
      ))}
    </svg>
  );
}

/** Whether a text with no box is outlined, so it reads over any picture. */
export function OutlineToggle({
  onChange,
  outline,
}: {
  onChange: (outline: boolean) => void;
  outline: boolean;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <OutlineGlyph outlined={isOn} />}
      isOn={outline}
      label={t("editor-island-text-outline")}
      onChange={onChange}
    />
  );
}

/** A capital I with its serifs, upright; turned on, it leans as italic
 * type does, about its middle, and turned off it stands up again. */
function ItalicGlyph({ italic }: { italic: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <motion.path
        animate={{ skewX: italic ? -16 : 0 }}
        d="M8 5h8M8 19h8M12 5v14"
        initial={false}
        style={{ originX: 0.5, originY: 0.5 }}
        transition={motionOf.move}
      />
    </svg>
  );
}

/** Whether a text is set in its family's italic, offered only where the
 * family has one. */
export function ItalicToggle({
  italic,
  onChange,
}: {
  italic: boolean;
  onChange: (italic: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <ItalicGlyph italic={isOn} />}
      isOn={italic}
      label={t("editor-island-text-italic")}
      onChange={onChange}
    />
  );
}

/** Whether a moving text's words come on one after another rather than
 * together. */
export function WordByWordToggle({
  onChange,
  revealWords,
}: {
  onChange: (revealWords: boolean) => void;
  revealWords: boolean;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <WordsGlyph oneByOne={isOn} />}
      isOn={revealWords}
      label={t("editor-island-text-word-by-word")}
      onChange={onChange}
    />
  );
}
