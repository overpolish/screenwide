// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef, useState } from "react";
import { useMove } from "react-aria";
import { Slider, SliderTrack } from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { SliderKnob } from "../../../../primitives/slider/slider";

import { comparedPlacement } from "./export-compare-placement";

import type {
  ToolPanelComparedFrame,
  ToolPanelExportCompare,
} from "../../tools/tool-panel-export";

/** How far a frame larger than the box sits from its centre at most. */
const clampOffset = (offset: number, frame: number, box: number) =>
  frame <= box ? (box - frame) / 2 : Math.min(0, Math.max(box - frame, offset));

/**
 * The picture as drawn on the start side of a divider and as the export
 * writes it on the end side, shown pixel for pixel on the screen so a
 * smaller or more compressed export shows its softness and blocking; the
 * exported picture is laid over the original's so the canvas both show
 * lines up, as a player shows a smaller video in the same space. The
 * divider is a slider, dragged or moved with the arrow keys; dragging the
 * picture looks at another part of it. Softened, with a spinner, while
 * newer pictures are made.
 */
export function ExportCompare({
  compare: { exported, failed, isUpdating, original },
}: {
  compare: ToolPanelExportCompare;
}) {
  const boxRef = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState({ height: 0, width: 0 });
  const [split, setSplit] = useState(50);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  useLayoutEffect(() => {
    const element = boxRef.current;
    if (!element) return;
    const observer = new ResizeObserver(() => {
      setBox({ height: element.clientHeight, width: element.clientWidth });
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);
  // One picture pixel to one screen pixel.
  const scale = 1 / window.devicePixelRatio;
  const width = (original?.width ?? 0) * scale;
  const height = (original?.height ?? 0) * scale;
  const left = clampOffset((box.width - width) / 2 + pan.x, width, box.width);
  const top = clampOffset(
    (box.height - height) / 2 + pan.y,
    height,
    box.height,
  );
  const { moveProps } = useMove({
    onMove: ({ deltaX, deltaY }) => {
      setPan((current) => ({
        x:
          clampOffset(
            (box.width - width) / 2 + current.x + deltaX,
            width,
            box.width,
          ) -
          (box.width - width) / 2,
        y:
          clampOffset(
            (box.height - height) / 2 + current.y + deltaY,
            height,
            box.height,
          ) -
          (box.height - height) / 2,
      }));
    },
  });
  // The exported picture is laid over the Original one by the canvas both
  // show, in the Original's pixels.
  const placed = (
    frame: ToolPanelComparedFrame,
    at = { height: frame.height, left: 0, top: 0, width: frame.width },
  ) => (
    <img
      alt=""
      className="pointer-events-none absolute max-w-none select-none"
      draggable={false}
      src={frame.src}
      style={{
        height: at.height * scale,
        insetInlineStart: left + at.left * scale,
        top: top + at.top * scale,
        width: at.width * scale,
      }}
    />
  );
  return (
    // The picture keeps its own left and right in every language.
    <div
      className="relative h-40 overflow-hidden rounded-control bg-fill"
      dir="ltr"
      ref={boxRef}
    >
      {original ? (
        <div
          {...moveProps}
          className={cn(
            "absolute inset-0 cursor-grab touch-none transition-[filter] active:cursor-grabbing",
            isUpdating && "blur-xs",
          )}
        >
          {placed(original)}
          {exported ? (
            <div
              className="absolute inset-0"
              style={{ clipPath: `inset(0 0 0 ${split.toString()}%)` }}
            >
              {placed(exported, comparedPlacement(original, exported))}
            </div>
          ) : null}
        </div>
      ) : null}
      {original && exported && !isUpdating ? (
        <>
          <div
            aria-hidden
            className="pointer-events-none absolute inset-y-0 w-px -translate-x-1/2 bg-fg"
            style={{ insetInlineStart: `${split.toString()}%` }}
          />
          <Slider
            aria-label={t("editor-export-compare-divider")}
            className="pointer-events-none absolute inset-0"
            onChange={setSplit}
            value={split}
          >
            <SliderTrack className="h-full w-full">
              <SliderKnob className="pointer-events-auto cursor-ew-resize" />
            </SliderTrack>
          </Slider>
        </>
      ) : null}
      {/* Both sides named in every state, on the island's black so they
          read over a bright picture as well as a dark one. */}
      <Badge className="pointer-events-none absolute start-inset-sm top-inset-sm bg-island/80 backdrop-blur-sm">
        {t("editor-export-compare-original")}
      </Badge>
      <Badge className="pointer-events-none absolute end-inset-sm top-inset-sm bg-island/80 backdrop-blur-sm">
        {t("editor-export-compare-export")}
      </Badge>
      {failed ? (
        <p className="absolute inset-x-0 bottom-inset-sm text-center text-caption text-fg-secondary">
          {t("editor-export-compare-failed")}
        </p>
      ) : isUpdating || !original ? (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <CircularProgress
            aria-label={t("editor-export-compare-making")}
            isIndeterminate
          />
        </div>
      ) : null}
    </div>
  );
}
