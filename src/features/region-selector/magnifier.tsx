// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence } from "motion/react";
import { useEffect, useMemo, useState } from "react";

import { AnimatedPixelMagnifier } from "../../components/shared/canvas-tools/pixel-magnifier";

import {
  magnifierCapturePoint,
  magnifierHandlePoint,
} from "./magnifier-geometry";
import { ResizeDirection } from "./types";

type RegionRect = { height: number; width: number; x: number; y: number };

type MagnifierProps = {
  regionRect: RegionRect;
  resizeDirection: ResizeDirection | undefined;
  screenshot: { height: number; pixels: ArrayBuffer; width: number };
};

export function Magnifier({
  regionRect,
  resizeDirection,
  screenshot,
}: MagnifierProps) {
  const [pointer, setPointer] = useState<{ x: number; y: number } | null>(null);

  const source = useMemo(() => {
    const expectedLength = screenshot.width * screenshot.height * 4;
    if (screenshot.pixels.byteLength !== expectedLength) return null;
    const pixels = new Uint8ClampedArray(screenshot.pixels);
    const canvas = document.createElement("canvas");
    canvas.width = screenshot.width;
    canvas.height = screenshot.height;
    const context = canvas.getContext("2d");
    if (!context) return null;
    context.putImageData(
      new ImageData(pixels, screenshot.width, screenshot.height),
      0,
      0,
    );
    return canvas;
  }, [screenshot]);

  const viewport = {
    height: window.innerHeight,
    width: window.innerWidth,
  };
  const position = magnifierHandlePoint(regionRect, resizeDirection, pointer);
  const capturePoint = magnifierCapturePoint(position, viewport, screenshot);

  useEffect(() => {
    const update = (event: MouseEvent) => {
      // React batches this with react-rnd's region update from the same native
      // event. Rendering derives the point from both current values, rather
      // than sampling the previous region edge from an event listener.
      setPointer({ x: event.clientX, y: event.clientY });
    };

    window.addEventListener("mousemove", update);
    return () => {
      window.removeEventListener("mousemove", update);
    };
  }, []);

  return (
    <AnimatePresence>
      {resizeDirection ? (
        <AnimatedPixelMagnifier
          className="pointer-events-none fixed"
          direction={resizeDirection}
          point={capturePoint}
          source={source}
          style={{
            height: 100,
            left: position.x,
            top: position.y,
            width: 100,
          }}
        />
      ) : null}
    </AnimatePresence>
  );
}
