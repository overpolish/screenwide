// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { type ReactNode, useId } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { t } from "../../../i18n/i18n";
import {
  motionDurationCss,
  motionDurations,
  motionEasings,
} from "../../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { Text } from "../../../primitives/text/text";

import { ModeIcon } from "./island-glyphs";
import { modeName } from "./island-names";

import type { IslandDisplay, IslandMode } from "./island-types";

// The parts the mode panel is built from: the modes, the displays, and a row
// for what a mode has chosen or still needs.

const MODES: IslandMode[] = ["screen", "window", "region", "none"];

/** The modes, the chosen one on a raised card that slides to the next one
 * chosen, as a pill group's knob does; its text follows the card's arrival.
 * A radio group: one stop for Tab, the arrow keys move between the modes. */
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
    <ToggleButtonGroup
      aria-label={t("recording-controls-mode")}
      className="grid grid-cols-4 gap-1"
      disallowEmptySelection
      onSelectionChange={(keys) => {
        for (const key of keys) onModeChange(key as IslandMode);
      }}
      selectedKeys={[mode]}
      selectionMode="single"
    >
      {MODES.map((item) => (
        <ToggleButton
          className={cn(
            "relative flex cursor-default flex-col items-center rounded-control py-2 transition-colors",
            mode === item
              ? "text-on-primary"
              : "text-fg-secondary data-[hovered]:bg-fill data-[hovered]:text-fg",
            focusStyles,
            elementFocusVisible,
          )}
          id={item}
          key={item}
          style={{
            transitionDuration: reducedMotion
              ? "0s"
              : motionDurationCss("state"),
          }}
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
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}

/** Each display by its picture, to choose one. Past two they go three
 * across, so a desk of them stays a row or two tall rather than pushing the
 * island down the screen. A lone display is centred at the size it has
 * beside a second, so the picture never hugs one side. One display is
 * chosen at a time, so it is a radio group, as the modes are. */
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
    <ToggleButtonGroup
      aria-label={t("recording-controls-displays")}
      className={cn(
        "grid gap-inline",
        displays.length > 2 ? "grid-cols-3" : "grid-cols-2",
      )}
      disallowEmptySelection
      onSelectionChange={(keys) => {
        for (const key of keys) onDisplayChange?.(Number(key));
      }}
      selectedKeys={[String(selectedDisplayId)]}
      selectionMode="single"
    >
      {displays.map((display) => (
        <ToggleButton
          className={cn(
            "group flex cursor-default flex-col gap-1 rounded-control text-start outline-none",
            displays.length === 1 &&
              "col-span-2 w-[calc((100%-var(--spacing-inline))/2)] justify-self-center",
          )}
          id={String(display.id)}
          key={display.id}
        >
          {/* The ring is the picture's: the chosen display's in the primary
              colour, the focused one's as focus is drawn everywhere. */}
          <span
            className={cn(
              "aspect-video w-full overflow-hidden rounded-control bg-fill ring-2 transition-shadow",
              display.id === selectedDisplayId
                ? "ring-primary"
                : "ring-transparent group-data-[hovered]:ring-fill-strong",
              "group-data-[focus-visible]:ring-3 group-data-[focus-visible]:ring-focus",
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
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
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
