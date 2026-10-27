// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FastForward, Hourglass, Pin, Scissors, Snail } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";

import { cn } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { speedLabel, speedOf } from "../model/speed";
import { DEAD_AIR_S, type Pause } from "../model/timeline-model";
import { type Editor } from "../model/use-editor";

import { Spring } from "./spring";
import { placed, type StripScale } from "./strip-scale";
import { useAltKey } from "./use-alt-key";

// The pauses between beats, each its own spring, which only ever stands for
// speed. Press one to choose it; double-click it to keep it as recorded, out
// of the squeeze; once chosen, drag its end to speed it up or slow it down,
// or with ⌥ held, to trim it. The squeeze from the end of the strip speeds
// up only the pauses left to it, so a deliberate pause can be kept while
// dead air elsewhere is hurried through.

/** The narrowest a pause shows its label at, in px. */
const LABEL_PX = 56;
/** How near its recorded length, in px, a dragged end snaps to keeping it. */
const FULL_SNAP_PX = 8;
/** The least width, in px, a pause can be pressed at. */
const PAUSE_HIT_PX = 14;

/** The width, in px, of the badge in `ref` while `isPresent`, kept as it
 * changes. */
function useBadgeWidth(
  ref: React.RefObject<HTMLSpanElement | null>,
  isPresent: boolean,
) {
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const badge = ref.current?.firstElementChild;
    if (!isPresent || !badge) return;
    const observer = new ResizeObserver(([entry]) => {
      setWidth(entry.borderBoxSize[0]?.inlineSize ?? 0);
    });
    observer.observe(badge);
    return () => {
      observer.disconnect();
    };
  }, [ref, isPresent]);
  return isPresent ? width : 0;
}

/** An outline's mask with a gap in the middle of its top edge as wide as a
 * badge of `badgeWidth` px with a little air: the top few px are shown
 * only outside the gap, the rest whole. */
const gapUnder = (badgeWidth: number): React.CSSProperties => {
  const half = `${String(badgeWidth / 2 + 3)}px`;
  return {
    maskImage: `linear-gradient(to right, #000 calc(50% - ${half}), transparent calc(50% - ${half}), transparent calc(50% + ${half}), #000 calc(50% + ${half})), linear-gradient(#000, #000)`,
    maskPosition: "0 0, 0 6px",
    maskRepeat: "no-repeat",
    maskSize: "100% 100%, 100% calc(100% - 6px)",
  };
};

function PauseBlock({
  editor,
  pause,
  scale,
  width,
}: {
  editor: Editor;
  pause: Pause;
  scale: StripScale;
  width: number;
}) {
  const ref = useRef<HTMLButtonElement>(null);
  // What the end's drag does, fixed as it is pressed.
  const dragRef = useRef<"retime" | "trim" | null>(null);
  const setting = editor.settings[pause.id] as
    Editor["settings"][string] | undefined;
  const { selection } = editor;
  const isSelected = selection?.kind === "pause" && selection.id === pause.id;
  const isDeadAir = pause.still > DEAD_AIR_S;
  const recorded = pause.end - pause.start;
  const plays = editor.map.pauseLength(pause);
  const px = Math.max(
    0,
    (scale.toX(pause.end) - scale.toX(pause.start)) * width,
  );
  // Squeezed narrower than a fingertip, it reaches over the edges of the
  // beats beside it, and sits above them there, so it can still be pressed.
  const isNarrow = px < PAUSE_HIT_PX;
  const look =
    setting?.kind === "keep" ? "kept" : isDeadAir ? "dead-air" : "plain";
  const speed = speedOf(recorded, plays);
  const isAlt = useAltKey(isSelected);
  const badgeRef = useRef<HTMLSpanElement>(null);
  const hasBadge = Boolean(setting) || isDeadAir || speed !== null;
  const badgeWidth = useBadgeWidth(badgeRef, hasBadge);
  // The outline: chosen, or dead air still to be dealt with. Where the
  // badge sits on its top edge the outline stops, so the badge's own fill
  // never shows it through.
  // A narrow pause is pressed across more than it is drawn; unchosen, its
  // outline keeps to the drawn width, off the beats beside it.
  const outline = isSelected
    ? "-inset-0.5 border-2 border-primary"
    : isDeadAir && !setting
      ? "inset-y-0 border border-dashed border-fg-tertiary"
      : null;
  const outlineInset = isSelected || !isNarrow ? 0 : (PAUSE_HIT_PX - px) / 2;
  const isBadgeShown = px > LABEL_PX || isSelected;

  return (
    <button
      aria-label={`Pause, ${recorded.toFixed(1)} s`}
      aria-pressed={isSelected}
      className={cn(
        "group absolute inset-y-1.5 cursor-default rounded-[6px] transition-colors",
        // Chosen, it sits over the beats beside it, so its end handle,
        // which reaches past its edge, can be taken hold of.
        isSelected
          ? "z-10 bg-selected"
          : cn("hover:bg-fill", isNarrow && "z-[5]"),
      )}
      // Chosen, like a beat, it takes the playhead to its start, so playing
      // from there shows how it plays now.
      onClick={() => {
        editor.select({ id: pause.id, kind: "pause" });
        editor.seek(pause.start);
      }}
      onDoubleClick={() => {
        editor.setPause(
          pause.id,
          setting?.kind === "keep" ? null : { kind: "keep" },
        );
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      ref={ref}
      style={
        isNarrow
          ? {
              insetInlineStart:
                scale.toX(pause.start) * width - (PAUSE_HIT_PX - px) / 2,
              width: PAUSE_HIT_PX,
            }
          : placed(scale, pause.start, pause.end)
      }
      type="button"
    >
      {outline ? (
        <span
          aria-hidden
          className={cn("pointer-events-none absolute rounded-[6px]", outline)}
          style={{
            ...(isSelected ? null : { insetInline: outlineInset }),
            ...(isBadgeShown && badgeWidth > 0 ? gapUnder(badgeWidth) : null),
          }}
        />
      ) : null}
      <Spring look={look} seconds={recorded} width={px} />
      {hasBadge ? (
        // How it plays: kept, faster or slower than recorded, or dead air
        // still to be dealt with. Squeezed too narrow to carry it, the
        // badge waits for the pointer or the selection, since the pauses
        // squeezed hardest are the ones whose speed matters most.
        <span
          className={cn(
            "pointer-events-none absolute inset-x-0 -top-3 z-10 flex justify-center transition-opacity",
            !isBadgeShown && "opacity-0 group-hover:opacity-100",
          )}
          ref={badgeRef}
        >
          <Badge>
            {setting?.kind === "keep" ? (
              <Pin aria-hidden />
            ) : speed === null ? (
              <Hourglass aria-hidden />
            ) : speed > 1 ? (
              <FastForward aria-hidden />
            ) : (
              <Snail aria-hidden />
            )}
            {speed === null ? null : speedLabel(speed)}
          </Badge>
        </span>
      ) : null}
      {isSelected ? (
        // Its end. Dragged, it retimes the pause: the spring stretches or
        // tightens, from four times as fast to half speed, and every frame
        // stays. Dragged with ⌥, it trims: what is dragged past is cut, with
        // a seam to put it back, and the rest plays as recorded.
        <span
          aria-hidden
          className={cn(
            "absolute -end-2 top-1/2 flex h-7 w-4 -translate-y-1/2 items-center justify-center",
            isAlt ? "cursor-col-resize" : "cursor-ew-resize",
          )}
          data-pause-end
          onDoubleClick={(event) => {
            event.stopPropagation();
          }}
          onPointerDown={(event) => {
            event.stopPropagation();
            event.currentTarget.setPointerCapture(event.pointerId);
            dragRef.current = event.altKey ? "trim" : "retime";
          }}
          onPointerMove={(event) => {
            const box = ref.current?.getBoundingClientRect();
            if (!dragRef.current || !box || box.width === 0) return;
            const share = Math.max(0, event.clientX - box.left) / box.width;
            if (dragRef.current === "trim") {
              // What is drawn up to the pointer is kept, played as
              // recorded, so a retimed pause's end stays under the pointer
              // as it goes back to real speed.
              editor.trimPause(pause.id, pause.start + plays * share);
              return;
            }
            const seconds = plays * share;
            // Drawn to within a few px of its recorded length, it is the
            // recorded pause again: kept, not retimed.
            const off = (Math.abs(recorded - seconds) / plays) * box.width;
            editor.setPause(
              pause.id,
              off < FULL_SNAP_PX ? { kind: "keep" } : { kind: "own", seconds },
            );
          }}
          onPointerUp={() => {
            dragRef.current = null;
          }}
        >
          {isAlt ? (
            <span className="flex size-5 items-center justify-center rounded-full bg-primary text-on-primary ring-2 ring-recess">
              <Scissors className="size-3" />
            </span>
          ) : (
            <span className="h-5 w-1.5 rounded-full bg-primary ring-2 ring-recess" />
          )}
        </span>
      ) : null}
    </button>
  );
}

export function Pauses({
  editor,
  scale,
  width,
}: {
  editor: Editor;
  scale: StripScale;
  width: number;
}) {
  // Where the picture moves by itself in a pause, as a band under its
  // spring: give the squeeze leaves alone, since it cannot tell a loading
  // bar from a preview playing, and that a retime of the pause hurries less.
  const moving = editor.map.pieces.filter(
    (piece) => piece.isMoving && !piece.beat && !piece.isCut,
  );
  return (
    <>
      {moving.map((piece) => (
        <span
          aria-hidden
          className="pointer-events-none absolute inset-y-4 rounded-full bg-fill-strong"
          key={piece.start}
          style={placed(scale, piece.start, piece.end)}
        />
      ))}
      {editor.map.pauses.map((pause) => (
        <PauseBlock
          editor={editor}
          key={pause.id}
          pause={pause}
          scale={scale}
          width={width}
        />
      ))}
    </>
  );
}
