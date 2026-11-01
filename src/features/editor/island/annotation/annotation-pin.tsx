// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EyeOff, Pin } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Alert } from "../../../../primitives/alert/alert";
import { Button } from "../../../../primitives/button/button";
import { IconToggleButton } from "../../../../primitives/button/icon-button";
import { IslandWorkLine } from "../island-work-line";

// Pinning an annotation to the content under it, which only a recording's
// screen annotations can do: a toggle at the row's end beside Animate, and
// while pinned, how tracking stands at the playhead at the island's foot,
// with the one answer that applies there and taking corrections away. Where
// the tracker is unsure of the content at the playhead, the note says why as
// a warning, so the place is checked and, if need be, put right with the
// same answers. The right-click menu's Tracking section does the same; this
// is the island's way to it.

/** What can be done to a pin from the island, as the menu's Tracking
 * section does. */
export type AnnotationPinAction = "clear" | "hide" | "pin" | "show" | "unpin";

/** Why the tracker is unsure of a pinned annotation's content: it lost it,
 * found it off the frame, or found it under a cover that stays put on
 * screen, such as a sticky header. */
export type AnnotationPinUnsure = "lost" | "off-frame" | "under-cover";

/** How a pinned annotation stands at the playhead. */
export type AnnotationPinView = {
  /** Saying its content went out of view here, or came back into view, or
   * neither where the playhead is outside it or on its pinned frame. */
  action: "hide" | "show" | null;
  /** Whether it was put anywhere by hand besides where it was pinned. */
  hasCorrections: boolean;
  /** How far its path is worked out, 0 to 1, while `state` is tracking;
   * null before the tracker has said. */
  progress: number | null;
  state: "following" | "out-of-view" | "tracking";
  /** Why the tracker is unsure of the content at the playhead, or null
   * where it is sure. */
  unsure: AnnotationPinUnsure | null;
};

const UNSURE_NOTES = {
  lost: "editor-island-annotation-pin-lost",
  "off-frame": "editor-island-annotation-pin-off-frame",
  "under-cover": "editor-island-annotation-pin-under-cover",
} as const;

export type AnnotationPinProps = {
  onPinAction?: (action: AnnotationPinAction) => void;
  /** Absent where it cannot be pinned (a screenshot's, the camera's, a pen's
   * next one), null while it is not. */
  pin?: AnnotationPinView | null;
};

/** Pinned or not: a pin, filled while it holds. Left out where the
 * annotation cannot be pinned. */
export function AnnotationPinToggle({ onPinAction, pin }: AnnotationPinProps) {
  if (pin === undefined) return null;
  return (
    <IconToggleButton
      aria-label={t("editor-island-annotation-pin")}
      isSelected={pin !== null}
      onChange={(isPinned) => {
        onPinAction?.(isPinned ? "pin" : "unpin");
      }}
    >
      {({ isSelected }) => (
        <Pin
          aria-hidden
          className={cn(
            "size-icon transition-colors",
            isSelected ? "fill-current" : "fill-transparent",
          )}
        />
      )}
    </IconToggleButton>
  );
}

/** How a pinned annotation's tracking stands at the playhead, at the
 * island's foot: worked out so far, following, or its content out of view,
 * or, where the tracker is unsure, why, as a warning; a note which takes the
 * section's width and adds none of its own, so its words wrap to the lines
 * above. Under it, the answers the playhead allows, one-shot actions at the
 * end of their own row, which wrap rather than widen the island where they
 * are long. */
export function AnnotationPinNote({ onPinAction, pin }: AnnotationPinProps) {
  if (!pin) return null;
  const answer = (action: AnnotationPinAction, label: string) => (
    <Button
      key={action}
      onPress={() => {
        onPinAction?.(action);
      }}
    >
      {label}
    </Button>
  );
  return (
    <div className="flex flex-col gap-separate">
      {pin.state === "tracking" ? (
        <IslandWorkLine
          label={t("editor-island-annotation-pin-tracking")}
          value={pin.progress ?? 0}
        />
      ) : pin.unsure ? (
        <Alert
          className="w-0 min-w-full px-inset-sm"
          color="warning"
          role="status"
        >
          {t(UNSURE_NOTES[pin.unsure])}
        </Alert>
      ) : (
        <Alert
          className="w-0 min-w-full px-inset-sm"
          icon={pin.state === "out-of-view" ? <EyeOff /> : <Pin />}
          role="status"
        >
          {pin.state === "out-of-view"
            ? t("editor-island-annotation-pin-out-of-view")
            : t("editor-island-annotation-pin-following")}
        </Alert>
      )}
      {pin.action !== null || pin.hasCorrections ? (
        <div className="flex w-0 min-w-full flex-wrap justify-end gap-separate">
          {pin.action === "hide"
            ? answer("hide", t("editor-island-annotation-pin-hide"))
            : null}
          {pin.action === "show"
            ? answer("show", t("editor-island-annotation-pin-show"))
            : null}
          {pin.hasCorrections
            ? answer("clear", t("editor-island-annotation-pin-clear"))
            : null}
        </div>
      ) : null}
    </div>
  );
}
