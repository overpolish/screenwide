// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { type Span } from "../timeline-scale";

import { type TimelineStripProps } from "./timeline-strip-props";

// The strip's two editing tools, as a video editor has them. The blade, in
// hand from its tool (B), splits the recording where it is pressed; a blade
// follows the pointer, so where it will split shows before it is pressed.
// The range tool, in hand from its tool (⇧R) or while ⌥ is held, which
// outranks the blade, draws a stretch over lulls and beats alike, chosen
// when let go, for its island to cut. The playhead follows the edge being
// drawn, so the picture shows the last frame the range takes in, and stays
// there once let go. A chosen lull's end keeps ⌥ for its own trim, so a
// press there is left to it. With the Export island making a GIF the strip
// is locked but for the range tool, which stays in hand and stops at the
// GIF's longest.

/** The shortest stretch, in seconds, a drag chooses; less is a slip. */
const LEAST_S = 0.1;
/** How far, in px, a press travels before it is a drag, as the strip's
 * other drags do. */
const PRESS_PX = 4;

export function useRangeCut({
  onChoose,
  scrub: { fractionOf, secondsAt },
  strip,
}: {
  /** Told the stretch a drag drew, once let go. */
  onChoose: (span: Span) => void;
  scrub: {
    /** Where the pointer is, as a share of the track's width. */
    fractionOf: (event: React.PointerEvent) => number;
    /** The recorded moment under the pointer. */
    secondsAt: (event: React.PointerEvent) => number;
  };
  strip: Pick<
    TimelineStripProps,
    | "gifRangeS"
    | "isAltHeld"
    | "isCutting"
    | "isLocked"
    | "isRanging"
    | "map"
    | "onSeek"
    | "onSplit"
  >;
}) {
  const { gifRangeS, isAltHeld, isLocked, map, onSeek, onSplit } = strip;
  const isRanging =
    gifRangeS !== undefined || (!isLocked && (strip.isRanging || isAltHeld));
  const isCutting = !isLocked && strip.isCutting && !isRanging;
  const fromRef = useRef<number | null>(null);
  const pressXRef = useRef(0);
  // Set by a press the tool took, so the click that ends it does not
  // choose what is under the pointer as well.
  const wasTakenRef = useRef(false);
  const [draft, setDraft] = useState<Span | null>(null);
  const [bladeX, setBladeX] = useState<number | null>(null);
  const isActive = isCutting || isRanging;

  /** Where a range drawn from `from` toward `to` may reach: a GIF's stops
   * at its longest in the edited recording. */
  const limit = (from: number, to: number) => {
    if (gifRangeS === undefined) return to;
    const at = map.toPlayed(from);
    const played = map.toPlayed(to);
    if (Math.abs(played - at) <= gifRangeS) return to;
    return map.toRecorded(at + Math.sign(played - at) * gifRangeS);
  };
  /** The edge `event` draws a range from `from` to, kept to `limit`. */
  const edgeAt = (event: React.PointerEvent) =>
    limit(fromRef.current ?? 0, secondsAt(event));
  const spanTo = (to: number): Span => {
    const from = fromRef.current ?? 0;
    return { end: Math.max(from, to), start: Math.min(from, to) };
  };

  return {
    bladeX: isCutting ? bladeX : null,
    draft,
    handlers: {
      onClickCapture: (event: React.MouseEvent) => {
        if (!wasTakenRef.current) return;
        wasTakenRef.current = false;
        event.stopPropagation();
      },
      onPointerDownCapture: (event: React.PointerEvent<HTMLElement>) => {
        const target = event.target as Element;
        if (!isActive || target.closest("[data-lull-end]")) return;
        // The band above the scenes scrubs, but for the range tool, which
        // draws from there too.
        if (!isRanging && target.closest("[data-scrub-band]")) return;
        event.stopPropagation();
        event.currentTarget.setPointerCapture(event.pointerId);
        fromRef.current = secondsAt(event);
        pressXRef.current = event.clientX;
        wasTakenRef.current = true;
        if (isRanging) {
          setDraft(spanTo(fromRef.current));
          onSeek(fromRef.current, "start");
        }
      },
      onPointerLeave: () => {
        setBladeX(null);
      },
      onPointerMoveCapture: (event: React.PointerEvent) => {
        setBladeX(fractionOf(event));
        if (fromRef.current === null) return;
        event.stopPropagation();
        if (isRanging) {
          const edge = edgeAt(event);
          setDraft(spanTo(edge));
          onSeek(edge, "move");
        }
      },
      onPointerUpCapture: (event: React.PointerEvent) => {
        const from = fromRef.current;
        if (from === null) return;
        event.stopPropagation();
        const edge = edgeAt(event);
        const span = spanTo(edge);
        fromRef.current = null;
        setDraft(null);
        const isPress = Math.abs(event.clientX - pressXRef.current) < PRESS_PX;
        if (isRanging) {
          onSeek(edge, "end");
          if (!isPress && span.end - span.start >= LEAST_S) onChoose(span);
        } else if (isPress) onSplit(from);
      },
    },
  };
}
