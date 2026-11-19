// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type Ref } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import {
  boundsAnchor,
  type PopupMenuAnchor,
} from "../../popup-panel/use-popup-menu";

// A cue's mark at the text column's end, in the room the column keeps there
// for marks (`pe-12`), standing over the field so it can be pressed; its
// layer (`CueMarks`) sets how far down, level with its word's line. It
// stands in from the column's edge past the grip that drags the edge
// (`w-3`, in `TeleprompterColumn`), so a press on it is never taken for the
// edge's. It is the cue's picture, tiny, so which cue it is can be told at a
// glance. Clicked, it opens the word's menu, as a right click on the word
// does; dragged, it takes the cue to another word, ringed and lifted while
// held. Like the words it stands beside, it is left out of the tab order and
// assistive technology.
export function CueMark({
  isDragged,
  onPress,
  ref,
  src,
  title,
}: {
  /** Being dragged, so drawn lifted. */
  isDragged: boolean;
  /** Pressed: `press` is where, `anchor` the mark's bounds, for a menu. */
  onPress: (
    press: { clientX: number; clientY: number },
    anchor: PopupMenuAnchor,
  ) => void;
  ref: Ref<HTMLButtonElement>;
  /** The cue's picture's address. */
  src: string;
  title: string;
}) {
  return (
    <button
      aria-label={t("teleprompter-cue-mark", { title })}
      className={cn(
        "absolute end-4 z-10 size-6 -translate-y-1/2 cursor-grab touch-none overflow-hidden rounded-full bg-fill-strong ring-1 ring-fill-strong transition-shadow hover:ring-2",
        isDragged && "cursor-grabbing shadow-lg ring-2 ring-primary",
      )}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        onPress(
          event,
          boundsAnchor(event.currentTarget.getBoundingClientRect()),
        );
      }}
      ref={ref}
      tabIndex={-1}
      type="button"
    >
      {/* A background rather than an image: WebKit offers Live Text on an
          image the pointer rests on, a button that has no place on a mark
          this small. */}
      <span
        aria-hidden
        className="block size-full bg-cover bg-center"
        style={{ backgroundImage: `url(${JSON.stringify(src)})` }}
      />
    </button>
  );
}
