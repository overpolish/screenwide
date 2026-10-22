// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AppWindow,
  type LucideIcon,
  Monitor,
  MonitorOff,
  SquareDashed,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { IconSwap } from "../../../primitives/icon-swap/icon-swap";
import {
  IslandButton,
  IslandOptions,
} from "../../../primitives/island/island-controls";

import { chosenName, INPUT_GLYPHS, inputName } from "./island-names";

import type { IslandInput, IslandInputKind, IslandMode } from "./island-types";

// The island's resting row is a row of one shape: every control a circle of
// the row's height. What is recorded shows as its picture; each input is a
// glyph that is its own meter. A control with choices behind it carries a
// small chevron beside it, plain on the island's black, which opens them;
// it sits just clear of its own control, never over it, and pairs stand
// further apart from each other, so no chevron reads as belonging to the
// control after it. Nothing on it changes at once: a glyph that changes
// crossfades in place, and the problem dot pops in and shrinks away.

/** How the problem dot shrinks away: the app's travel motion, no settle. */
const dotAway = {
  duration: motionDurations.travel,
  ease: motionEasings.out,
} as const;

/** How the problem dot pops in: a spring with a little settle, quicker than
 * travel, so it catches the eye without lingering. */
const dotIn = { bounce: 0.5, duration: 0.35, type: "spring" } as const;

export function InputGlyph({
  input,
  isOpen,
  kind,
  onOpen,
  onToggle,
}: {
  input: IslandInput;
  isOpen: boolean;
  kind: IslandInputKind;
  onOpen: () => void;
  onToggle: () => void;
}) {
  const { off: Off, on: On } = INPUT_GLYPHS[kind];
  // Off, an input's problem no longer stands in the way, as the warning
  // line already treats it.
  const hasProblem =
    input.isOn && (Boolean(input.warning) || Boolean(input.isLocked));
  return (
    <IslandOptions
      isOpen={isOpen}
      label={t("recording-controls-island-input-options", {
        input: inputName(kind),
      })}
      onOpen={onOpen}
    >
      <IslandButton
        aria-label={inputName(kind)}
        aria-pressed={input.isOn}
        isOpen={isOpen}
        onPress={onToggle}
        title={chosenName(input) ?? inputName(kind)}
      >
        <IconSwap swapKey={input.isOn ? "on" : "off"}>
          {input.isOn ? (
            <LevelFill icon={On} level={input.level} />
          ) : (
            <span className="flex text-fg-tertiary">
              <Off aria-hidden className="size-icon" />
            </span>
          )}
        </IconSwap>
        <ProblemDot isShown={hasProblem} />
      </IslandButton>
    </IslandOptions>
  );
}

/** An input's glyph as its own meter: dim, with its level filling it from
 * the bottom; the camera, which has no level, is simply full. Colours sit on
 * the wrappers, so each glyph takes one alpha as a whole (see `.lucide` in
 * index.css), and the dim part and the level are clipped apart so no pixel
 * is painted twice. Fills a positioned control. */
export function LevelFill({
  icon: Icon,
  level,
}: {
  icon: LucideIcon;
  level?: number;
}) {
  const fill = level === undefined ? 1 : 0.18 + level * 0.82;
  return (
    <>
      <span
        aria-hidden
        className="absolute inset-0 flex items-center justify-center text-fg-tertiary"
        style={{ clipPath: `inset(0 0 ${String(fill * 100)}% 0)` }}
      >
        <Icon className="size-icon" />
      </span>
      <span
        aria-hidden
        className="absolute inset-0 flex items-center justify-center text-fg"
        style={{ clipPath: `inset(${String((1 - fill) * 100)}% 0 0 0)` }}
      >
        <Icon className="size-icon" />
      </span>
    </>
  );
}

/** The mark on a control whose part holds something that stops recording,
 * seen whichever part is open. It pops in with a little settle, and shrinks
 * away once the problem is gone. */
function ProblemDot({ isShown }: { isShown: boolean }) {
  return (
    <AnimatePresence initial={false}>
      {isShown ? (
        <motion.span
          animate={{ opacity: 1, scale: 1 }}
          aria-hidden
          className="absolute end-1 top-1 size-1.5 rounded-full bg-error ring-2 ring-island"
          exit={{ opacity: 0, scale: 0, transition: dotAway }}
          initial={{ opacity: 0, scale: 0 }}
          key="problem"
          transition={dotIn}
        />
      ) : null}
    </AnimatePresence>
  );
}

const MODE_ICONS: Record<IslandMode, LucideIcon> = {
  none: MonitorOff,
  region: SquareDashed,
  screen: Monitor,
  window: AppWindow,
};

export function ModeIcon({
  className,
  mode,
}: {
  mode: IslandMode;
  className?: string;
}) {
  const Icon = MODE_ICONS[mode];
  return <Icon aria-hidden className={className} />;
}

/** What will be recorded, as its picture: the chosen display's still or
 * window's app icon, or else the mode's icon. Raised, as the one control
 * that names a choice rather than switching something; its name is in its
 * tooltip and in the part it opens. It carries the problem dot when what is
 * chosen cannot be recorded. */
export function SourceChip({
  hasProblem,
  isOpen,
  label,
  mode,
  onPress,
  picture,
}: {
  hasProblem: boolean;
  isOpen: boolean;
  label: string;
  mode: IslandMode;
  onPress: () => void;
  picture?: { kind: "app" | "display"; src: string };
}) {
  return (
    <IslandButton
      aria-expanded={isOpen}
      aria-label={t("recording-controls-island-source")}
      isRaised
      onPress={onPress}
      title={label}
    >
      <IconSwap swapKey={picture ? `${picture.kind} ${picture.src}` : mode}>
        {picture ? (
          // A display's still fills the circle; an app's icon sits in it.
          <img
            alt=""
            className={
              picture.kind === "display"
                ? "size-full object-cover"
                : "size-icon-lg object-contain"
            }
            src={picture.src}
          />
        ) : (
          <ModeIcon className="size-icon" mode={mode} />
        )}
      </IconSwap>
      <ProblemDot isShown={hasProblem} />
    </IslandButton>
  );
}
