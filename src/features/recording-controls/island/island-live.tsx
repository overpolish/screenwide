// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play, Square, Trash2, X } from "lucide-react";
import { motion, PresenceContext, useReducedMotion } from "motion/react";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import {
  IslandButton,
  IslandPrimaryButton,
} from "../../../primitives/island/island-controls";
import { Text } from "../../../primitives/text/text";

// The island while it records: a countdown that can be pressed to cancel,
// then a dot that pulses beside the time, with the controls opening under
// the pointer; paused, the dot goes grey and still and the controls stay.

/** The seconds before recording, each a ring that drains around its number
 * as the number lands; pressed, it calls the recording off. */
export function Countdown({
  count,
  onCancel,
}: {
  count: number;
  onCancel?: () => void;
}) {
  return (
    <button
      aria-label={t("recording-controls-cancel-countdown")}
      className="relative flex size-island-control items-center justify-center"
      onClick={onCancel}
      type="button"
    >
      {/* Each second's ring and number animate as they mount, which an
          AnimatePresence with initial={false} above would block, leaving an
          empty ring; the countdown is never an entrance, so it opts out. */}
      <PresenceContext value={null}>
        <svg
          aria-hidden
          className="absolute inset-0 -rotate-90"
          viewBox="0 0 32 32"
        >
          <circle
            className="stroke-fill"
            cx={16}
            cy={16}
            fill="none"
            r={14}
            strokeWidth={2}
          />
          <motion.circle
            animate={{ pathLength: 0 }}
            className="stroke-primary"
            cx={16}
            cy={16}
            fill="none"
            initial={{ pathLength: 1 }}
            key={count}
            r={14}
            strokeLinecap="round"
            strokeWidth={2}
            // One second, evenly, as the second it counts.
            transition={{ duration: 1, ease: "linear" }}
          />
        </svg>
        <motion.span
          animate={{ opacity: 1, scale: 1 }}
          className="text-label font-semibold tabular-nums"
          initial={{ opacity: 0, scale: 1.4 }}
          key={`n${String(count)}`}
        >
          {count}
        </motion.span>
      </PresenceContext>
    </button>
  );
}

/** The recording dot, with a faint, slow ripple while it records; paused,
 * the dot greys and the ripple stops. With reduced motion there is no
 * ripple. */
export function BreathingDot({ isPaused }: { isPaused: boolean }) {
  const reducedMotion = useReducedMotion();
  return (
    <span
      aria-hidden
      className="relative flex size-icon items-center justify-center"
    >
      {/* The pulse is the dot's whole life, not an entrance, so it opts out
          of any AnimatePresence above that would block it from starting.
          Each ripple fades in from nothing, peaks faintly partway out and
          fades away as it spreads, so nothing ever appears at once. */}
      <PresenceContext value={null}>
        {isPaused || reducedMotion ? null : (
          <motion.span
            animate={{ opacity: [0, 0.45, 0], scale: [1, 1.6, 2.2] }}
            className="absolute size-2 rounded-full bg-error"
            transition={{ duration: 2, ease: "easeInOut", repeat: Infinity }}
          />
        )}
      </PresenceContext>
      <span
        className={cn(
          "relative size-2 rounded-full transition-colors",
          isPaused ? "bg-fg-secondary" : "bg-error",
        )}
      />
    </span>
  );
}

/** A live control: quieter than the row's at rest, as the time is what the
 * island shows while recording. */
const LIVE = "text-fg-secondary enabled:hover:text-fg";

export function LiveControls({
  isPaused,
  onDiscard,
  onPause,
  onResume,
  onStop,
}: {
  isPaused: boolean;
  onDiscard?: () => void;
  onPause?: () => void;
  onResume?: () => void;
  onStop?: () => void;
}) {
  return (
    <span className="flex items-center gap-1">
      <IslandButton
        aria-label={
          isPaused
            ? t("recording-controls-resume")
            : t("recording-controls-pause")
        }
        className={LIVE}
        onPress={isPaused ? onResume : onPause}
      >
        {isPaused ? (
          <Play aria-hidden className="size-icon" />
        ) : (
          <Pause aria-hidden className="size-icon" />
        )}
      </IslandButton>
      {/* Discarding sits beside stopping, so it asks twice: the bin gives
          way to a check, and only that discards. */}
      <ConfirmActionButton
        className="size-island-control rounded-full"
        confirmLabel={t("recording-controls-confirm-discard")}
        icon={<Trash2 />}
        label={t("recording-controls-discard")}
        onConfirm={onDiscard}
        variant="ghost"
      />
      {/* Stop is Record's own button, so the pair reads as one action. */}
      <IslandPrimaryButton
        aria-label={t("recording-controls-stop")}
        onPress={onStop}
      >
        <Square aria-hidden className="size-icon" />
      </IslandPrimaryButton>
    </span>
  );
}

/** The capture getting going, after any countdown: always shown, however
 * briefly, and still possible to call off. The spinner sits in a slot the
 * size of the cancel button, so the two glyphs stand equally far from the
 * island's ends and the word sits centred between them. */
export function Starting({ onCancel }: { onCancel?: () => void }) {
  return (
    <span className="flex items-center gap-1">
      <span className="flex size-island-control items-center justify-center">
        <CircularProgress
          aria-label={t("recording-controls-starting-label")}
          isIndeterminate
          size="small"
        />
      </span>
      <Text as="span" role="status" variant="label">
        {t("recording-controls-starting")}
      </Text>
      <IslandButton
        aria-label={t("recording-controls-cancel-starting")}
        className={LIVE}
        onPress={onCancel}
      >
        <X aria-hidden className="size-icon" />
      </IslandButton>
    </span>
  );
}

/** The recording being saved. Its spinner sits in the same slot as
 * Starting's, and the word ends as far from the island's end as the spinner
 * starts from the other, so the two read alike. */
export function Finishing() {
  return (
    <span className="flex items-center gap-1 pe-2">
      <span className="flex size-island-control items-center justify-center">
        <CircularProgress
          aria-label={t("recording-controls-finishing-label")}
          isIndeterminate
          size="small"
        />
      </span>
      <Text as="span" role="status" variant="label">
        {t("recording-controls-finishing")}
      </Text>
    </span>
  );
}
