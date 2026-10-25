// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { cn } from "../../../../lib/styling";

import { paintScene, PICTURE, sceneFonts } from "./scene-kit";
import { type PreviewTool, TOOL_SCENES } from "./tool-scenes";
import { useNativeToolPreview } from "./use-native-tool-preview";

// A tool at work in Tools, drawn by the app itself rather than recreated in
// the page. The box paints the screen the tool works on. In the app, the
// Ruler and OCR draw that screen themselves, so the box turns transparent
// and their renderer shows through it; Annotate draws on glass laid over
// the painted screen. Switched off, or outside the app, the box shows the
// screen alone, drained of colour when off, as the other tools' pictures
// do. The screen is wider than the box and centred in it, so it is never
// repainted as the window resizes.

export function NativeToolScreen({
  isOn,
  look,
  tool,
}: {
  isOn: boolean;
  tool: PreviewTool;
  /** Annotate's colour and whether its annotations are kept. */
  look?: { color: string; keep: boolean };
}) {
  const ref = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const scene = TOOL_SCENES[tool];

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    let current = true;
    void sceneFonts().then(() => {
      if (current) paintScene(canvas, scene);
    });
    return () => {
      current = false;
    };
  }, [scene]);

  const isNative = useNativeToolPreview({ isOn, look, ref, scene, tool });
  const showsNative = isNative && isOn && !scene.glass;

  return (
    <div
      aria-hidden
      className={cn(
        "relative overflow-hidden rounded-control",
        !showsNative && "bg-recess",
      )}
      dir="ltr"
      inert
      ref={ref}
      style={{ height: PICTURE.height }}
    >
      <canvas
        className={cn(
          "absolute top-0 max-w-none transition-[filter,opacity]",
          showsNative && "opacity-0",
          !isOn && "opacity-50 grayscale",
        )}
        ref={canvasRef}
        // Centred however much wider than the box it is; the box is
        // left to right, so the start is the left.
        style={{
          height: PICTURE.height,
          insetInlineStart: "50%",
          transform: "translateX(-50%)",
          width: PICTURE.width,
        }}
      />
    </div>
  );
}
