// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { Ref, useEffect, useLayoutEffect, useRef, useState } from "react";
import { useFocusRing, useMove } from "react-aria";
import { Slider, SliderTrack } from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { motionDurationCss } from "../../../../lib/motion";
import {
  PictureCorner,
  pictureTones,
  PictureTones,
} from "../../../../lib/picture-tone";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { SliderKnob } from "../../../../primitives/slider/slider";
import { Text } from "../../../../primitives/text/text";

/** A frame to compare, at its own pixel size. */
export type ComparedFrame = { height: number; src: string; width: number };

type ExportPreviewCompareProps = {
  /** The frame as the export writes it, drawn at the Original frame's size,
   * as a player shows a smaller video in the same space. */
  exported: ComparedFrame | null;
  failed: boolean;
  isUpdating: boolean;
  /** The frame at Original, shown on the start side of the divider. */
  original: ComparedFrame | null;
  className?: string;
};

/** How far one arrow key press moves the frame, in CSS pixels. */
const KEY_STEP = 24;

/** How long the frame rests before the picture under the badges is read
 * again, so a drag reads it once where it stops. */
const TONE_DELAY_MS = 150;

const clampOffset = (offset: number, frame: number, viewport: number) =>
  frame <= viewport
    ? (viewport - frame) / 2
    : Math.min(0, Math.max(viewport - frame, offset));

/**
 * The frame under the playhead at Original and as exported, one each side
 * of a divider, shown pixel for pixel on the screen so a smaller or more
 * compressed export shows its softness and blocking. Dragged, or moved
 * with the arrow keys, to look at another part of the frame.
 */
export function ExportPreviewCompare({
  className,
  exported,
  failed,
  isUpdating,
  original,
}: ExportPreviewCompareProps) {
  const viewportRef = useRef<HTMLDivElement>(null);
  const [viewport, setViewport] = useState({ height: 0, width: 0 });
  const [split, setSplit] = useState(50);
  // Where the frame's centre sits from the viewport's, in CSS pixels.
  const [pan, setPan] = useState({ x: 0, y: 0 });

  useLayoutEffect(() => {
    const element = viewportRef.current;
    if (!element) return;
    const observer = new ResizeObserver(() => {
      setViewport({
        height: element.clientHeight,
        width: element.clientWidth,
      });
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);

  // One frame pixel to one screen pixel.
  const scale = 1 / window.devicePixelRatio;
  const frameWidth = (original?.width ?? 0) * scale;
  const frameHeight = (original?.height ?? 0) * scale;
  // Where the frame's corner sits with no pan: centred in the viewport.
  const centredX = (viewport.width - frameWidth) / 2;
  const centredY = (viewport.height - frameHeight) / 2;
  const left = clampOffset(centredX + pan.x, frameWidth, viewport.width);
  const top = clampOffset(centredY + pan.y, frameHeight, viewport.height);
  const { moveProps } = useMove({
    onMove: ({ deltaX, deltaY, pointerType }) => {
      const step = pointerType === "keyboard" ? KEY_STEP : 1;
      // Clamped as it goes, so dragging past an edge and back moves the
      // frame straight away.
      setPan((current) => ({
        x:
          clampOffset(
            centredX + current.x + deltaX * step,
            frameWidth,
            viewport.width,
          ) - centredX,
        y:
          clampOffset(
            centredY + current.y + deltaY * step,
            frameHeight,
            viewport.height,
          ) - centredY,
      }));
    },
  });
  const { focusProps, isFocusVisible } = useFocusRing();
  const transitionDuration = useReducedMotion()
    ? "0s"
    : motionDurationCss("state");

  // The badges take the appearance of the part of the frame beneath them,
  // as a project card's do, read once the frame has loaded and settled.
  const originalRef = useRef<HTMLImageElement>(null);
  const [loadedSrc, setLoadedSrc] = useState<string | null>(null);
  const [tones, setTones] = useState<PictureTones | null>(null);
  useEffect(() => {
    const image = originalRef.current;
    if (!image || loadedSrc === null || frameWidth === 0) return;
    const timer = window.setTimeout(() => {
      setTones(
        pictureTones(image, "ltr", {
          height: Math.min(viewport.height, frameHeight) / scale,
          width: Math.min(viewport.width, frameWidth) / scale,
          x: Math.max(0, -left) / scale,
          y: Math.max(0, -top) / scale,
        }),
      );
    }, TONE_DELAY_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [frameHeight, frameWidth, left, loadedSrc, scale, top, viewport]);
  const appearanceOver = (corner: PictureCorner) => {
    const tone = tones?.[corner];
    return tone ? `appearance-${tone}` : undefined;
  };

  const placed = (frame: ComparedFrame, ref?: Ref<HTMLImageElement>) => (
    <img
      alt=""
      className="pointer-events-none absolute max-w-none select-none"
      // Lets the badges read the pixels under them; the asset protocol
      // allows it for the window's own origin.
      crossOrigin="anonymous"
      draggable={false}
      onLoad={
        ref
          ? () => {
              setLoadedSrc(frame.src);
            }
          : undefined
      }
      ref={ref}
      src={frame.src}
      style={{
        height: frameHeight,
        insetInlineStart: left,
        top,
        width: frameWidth,
      }}
    />
  );

  return (
    // The picture keeps its own left and right in every language.
    <div
      className={cn(
        "relative overflow-hidden rounded-panel bg-fill-tertiary",
        className,
      )}
      dir="ltr"
      ref={viewportRef}
    >
      {original ? (
        <>
          <div
            {...moveProps}
            {...focusProps}
            aria-label={t("editor-export-preview-frame")}
            className={cn(
              "absolute inset-0 cursor-grab touch-none ring-inset active:cursor-grabbing",
              focusStyles,
              elementFocusVisible,
            )}
            data-focus-visible={isFocusVisible || undefined}
            role="group"
            tabIndex={0}
          >
            {/* Softened while newer frames are made, so a changed option
                reads as on its way rather than already shown. */}
            <div
              className={cn(
                "absolute inset-0 transition-[filter]",
                isUpdating && "blur-xs",
              )}
              style={{ transitionDuration }}
            >
              {placed(original, originalRef)}
              {exported ? (
                <div
                  className="absolute inset-0"
                  style={{ clipPath: `inset(0 0 0 ${split.toString()}%)` }}
                >
                  {placed(exported)}
                </div>
              ) : null}
            </div>
          </div>
          {/* Put away while newer frames are made: the loader takes the
              middle, and there is nothing current to compare. */}
          {exported && !isUpdating ? (
            <>
              <div
                aria-hidden
                className="pointer-events-none absolute inset-y-0 w-px -translate-x-1/2 bg-control-solid-fill"
                style={{ insetInlineStart: `${split.toString()}%` }}
              />
              <Slider
                aria-label={t("editor-export-preview-divider")}
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
          <Badge
            className={cn(
              "pointer-events-none absolute start-control top-control backdrop-blur-sm",
              appearanceOver("topStart"),
            )}
          >
            {t("editor-export-preview-original")}
          </Badge>
          {exported ? (
            <Badge
              className={cn(
                "pointer-events-none absolute end-control top-control backdrop-blur-sm",
                appearanceOver("topEnd"),
              )}
            >
              {t("editor-export-preview-export")}
            </Badge>
          ) : null}
          {failed ? (
            <Badge
              className={cn(
                "pointer-events-none absolute end-control bottom-control backdrop-blur-sm",
                appearanceOver("bottomEnd"),
              )}
            >
              {t("editor-export-preview-failed")}
            </Badge>
          ) : null}
        </>
      ) : failed ? (
        <div className="absolute inset-0 flex items-center justify-center p-section">
          <Text variant="subheadline">{t("editor-export-preview-failed")}</Text>
        </div>
      ) : null}
      {isUpdating || (!original && !failed) ? (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <CircularProgress
            aria-label={t("editor-export-preview-making")}
            isIndeterminate
          />
        </div>
      ) : null}
    </div>
  );
}
