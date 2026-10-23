// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { type ReactNode, useId } from "react";

import { t } from "../../../i18n/i18n";
import {
  motionDurationCss,
  motionDurations,
  motionEasings,
} from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { Text } from "../../../primitives/text/text";

import { ModeIcon } from "./island-glyphs";
import { modeName } from "./island-names";

import type { IslandDisplay, IslandMode } from "./island-types";

// The parts the mode panel is built from: the modes, the displays, and a row
// for what a mode has chosen or still needs.

const MODES: IslandMode[] = ["screen", "window", "region", "none"];

/** The modes, the chosen one on a raised card that slides to the next one
 * chosen, as a pill group's knob does; its text follows the card's arrival. */
export function ModeCards({
  mode,
  onModeChange,
}: {
  mode: IslandMode;
  onModeChange: (mode: IslandMode) => void;
}) {
  const cardId = useId();
  const reducedMotion = useReducedMotion();
  return (
    <div
      aria-label={t("recording-controls-mode")}
      className="grid grid-cols-4 gap-1"
      role="radiogroup"
    >
      {MODES.map((item) => (
        <button
          aria-checked={mode === item}
          className={cn(
            "relative flex flex-col items-center rounded-control py-2 transition-colors",
            mode === item
              ? "text-on-primary"
              : "text-fg-secondary hover:bg-fill hover:text-fg",
          )}
          key={item}
          onClick={() => {
            onModeChange(item);
          }}
          role="radio"
          style={{
            transitionDuration: reducedMotion
              ? "0s"
              : motionDurationCss("state"),
          }}
          type="button"
        >
          {mode === item ? (
            <motion.span
              aria-hidden
              className="absolute inset-0 rounded-[inherit] bg-primary primary-raised"
              layoutId={`island-mode-${cardId}`}
              transition={{
                duration: reducedMotion ? 0 : motionDurations.travel,
                ease: motionEasings.out,
              }}
            />
          ) : null}
          {/* Above the card in every mode, not just its own, so the card
              crossing a mode on its way does not cover its name. */}
          <span className="relative z-10 flex flex-col items-center gap-1">
            <ModeIcon className="size-icon" mode={item} />
            <Text
              as="span"
              className={mode === item ? "text-on-primary" : undefined}
            >
              {modeName(item)}
            </Text>
          </span>
        </button>
      ))}
    </div>
  );
}

/** Each display by its picture, to choose one. Past two they go three
 * across, so a desk of them stays a row or two tall rather than pushing the
 * island down the screen. */
export function DisplayGrid({
  displays,
  onDisplayChange,
  selectedDisplayId,
}: {
  displays: IslandDisplay[];
  selectedDisplayId: number;
  onDisplayChange?: (id: number) => void;
}) {
  return (
    <div
      className={cn(
        "grid gap-inline",
        displays.length > 2 ? "grid-cols-3" : "grid-cols-2",
      )}
    >
      {displays.map((display) => (
        <button
          aria-pressed={display.id === selectedDisplayId}
          className="flex flex-col gap-1 text-start"
          key={display.id}
          onClick={() => onDisplayChange?.(display.id)}
          type="button"
        >
          <span
            className={cn(
              "aspect-video w-full overflow-hidden rounded-control bg-fill ring-2 transition-shadow",
              display.id === selectedDisplayId
                ? "ring-primary"
                : "ring-transparent hover:ring-fill-strong",
            )}
          >
            {display.thumbnail ? (
              <img
                alt=""
                className="size-full object-cover"
                src={display.thumbnail}
              />
            ) : null}
          </span>
          <Text className="truncate text-center" dir="auto">
            {display.label}
          </Text>
        </button>
      ))}
    </div>
  );
}

/** A mode's icon in the space a chosen thing's picture takes. */
export function ModePicture({ mode }: { mode: IslandMode }) {
  return (
    <span className="flex size-8 shrink-0 items-center justify-center text-fg-secondary">
      <ModeIcon className="size-icon-lg" mode={mode} />
    </span>
  );
}

/** What a mode has chosen: its picture, what it is and where it is from,
 * and a button to choose again on the screen. */
export function ChosenRow({
  action,
  detail,
  isTitleDirectional = true,
  onAction,
  picture,
  title,
}: {
  detail: string;
  picture: ReactNode;
  title: string;
  /** Changes what was chosen; none where it is changed in place, as a
   * region is on the screen. */
  action?: string;
  /** A name to show in its own direction; false for figures. */
  isTitleDirectional?: boolean;
  onAction?: () => void;
}) {
  return (
    <div className="flex items-center gap-separate rounded-control bg-fill px-inset py-separate">
      {picture}
      <div className="flex min-w-0 grow flex-col">
        <Text
          className="truncate tabular-nums"
          dir={isTitleDirectional ? "auto" : undefined}
        >
          {title}
        </Text>
        <Text className="truncate" dir="auto" variant="caption">
          {detail}
        </Text>
      </div>
      {action ? (
        <Button className="shrink-0" onPress={onAction}>
          {action}
        </Button>
      ) : null}
    </div>
  );
}

/** What a mode still needs, and the button that sets about it, where it is
 * not done on the screen itself. */
export function PromptRow({
  action,
  onAction,
  prompt,
}: {
  prompt: string;
  action?: string;
  onAction?: () => void;
}) {
  return (
    <div className="flex items-center justify-between gap-separate rounded-control bg-fill px-inset py-separate">
      <Text>{prompt}</Text>
      {action ? (
        <Button className="shrink-0" onPress={onAction}>
          {action}
        </Button>
      ) : null}
    </div>
  );
}
