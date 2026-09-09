// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera } from "lucide-react";
import { Ref } from "react";

import { cn } from "../../lib/styling";

import {
  CameraPreviewDimensions,
  cameraPreviewFitClassName,
} from "./camera-preview-fit";

/** The stage a camera without a frame yet is drawn on, so the thumbnail does
 * not change shape the moment the first frame arrives. */
const FALLBACK_ASPECT_RATIO = 16 / 9;

type CameraThumbnailProps = {
  canvasRef: Ref<HTMLCanvasElement>;
  /** The shape the camera is sending, or is about to: the stage takes its
   * aspect ratio from this rather than from the canvas, whose intrinsic size
   * is 300 × 150 until a frame sizes it. */
  frameSize: CameraPreviewDimensions | null;
  hasFrame: boolean;
  "aria-label"?: string;
  /** Sets the stage's width; its height follows the aspect ratio. */
  className?: string;
  isDimmed?: boolean;
};

/**
 * A live camera preview at thumbnail size: the frame fitted whole into a stage
 * of the camera's own shape, with a glyph standing in until it arrives.
 */
export function CameraThumbnail({
  "aria-label": ariaLabel,
  canvasRef,
  className,
  frameSize,
  hasFrame,
  isDimmed = false,
}: CameraThumbnailProps) {
  return (
    <span
      className={cn(
        "relative flex items-center justify-center overflow-hidden rounded-sm bg-fill-quaternary",
        className,
      )}
      style={{
        aspectRatio: frameSize
          ? frameSize.width / frameSize.height
          : FALLBACK_ASPECT_RATIO,
      }}
    >
      <canvas
        aria-hidden={ariaLabel === undefined}
        aria-label={ariaLabel}
        className={cn(
          "pointer-events-none block shrink-0 transition-opacity",
          frameSize
            ? cameraPreviewFitClassName(frameSize)
            : "max-h-full max-w-full",
          isDimmed && "opacity-50",
        )}
        hidden={!hasFrame}
        ref={canvasRef}
      />
      {hasFrame ? null : (
        // Sized from the stage rather than from the icon scale: the same
        // thumbnail is drawn at 48px in the dock and at 20px in the bar's
        // inputs row, and an ancestor's glyph sizing would overflow the
        // smaller one.
        <Camera
          className="absolute text-content-fg-tertiary"
          style={{ height: "auto", width: "50%" }}
        />
      )}
    </span>
  );
}

export type { CameraThumbnailProps };
