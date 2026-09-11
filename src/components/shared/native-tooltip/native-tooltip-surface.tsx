// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ref } from "react";

import { Keyboard } from "../../base/keyboard/keyboard";
import { Text } from "../../base/text/text";
import { tooltipShapeClassName } from "../../base/tooltip/tooltip";

import type { NativeTooltipContent } from "./api";

/**
 * What a tooltip says: the label, and the shortcut that does the same thing
 * drawn as a keycap beside it.
 *
 * Shared by the tooltip window and the in-page tooltip it falls back to, so a
 * tooltip reads the same however it is drawn.
 */
export function NativeTooltipLabel({
  content,
}: {
  content: NativeTooltipContent;
}) {
  return (
    <span className="flex items-center gap-control-inset">
      {content.label}
      {content.shortcut ? <Keyboard>{content.shortcut}</Keyboard> : null}
    </span>
  );
}

/**
 * A tooltip drawn as the whole of its own window.
 *
 * The panel is the window: it is measured, the window is sized to it, and
 * there is nothing else on the page. It takes its look from the in-page
 * tooltip's own classes rather than restating them.
 */
export function NativeTooltipSurface({
  content,
  ref,
}: {
  content: NativeTooltipContent;
  /** Measured by the window so it can fit itself to the words. */
  ref?: Ref<HTMLDivElement>;
}) {
  return (
    // Shrink-wrapped, so what is measured is the size the words need rather
    // than the size the window happens to be.
    <div className="inline-flex w-max max-w-72" ref={ref}>
      <Text
        as="span"
        // Glass: the window's material is the background, as a native
        // tooltip's is, so the surface paints only its shape and text.
        className={`window-surface ${tooltipShapeClassName} inline-flex`}
        variant="subheadline"
      >
        <NativeTooltipLabel content={content} />
      </Text>
    </div>
  );
}
