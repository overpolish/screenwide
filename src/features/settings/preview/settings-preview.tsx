// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { cn } from "../../../lib/styling";

import { paintStill } from "./paint-pictures";
import {
  pictureSize,
  sceneOf,
  type SettingsPreviewScene,
} from "./scenes/preview-scenes";
import { useDesktopWallpaper } from "./use-desktop-wallpaper";
import { useDisplayNotches } from "./use-display-notches";
import { useSettingsPreview } from "./use-settings-preview";
import { useSystemCursor } from "./use-system-cursor";

// A setting at work, drawn by the app itself rather than recreated in the
// page. The box paints the desktop the scene works on, on the person's own
// desktop picture. In the app, the Ruler and OCR draw that desktop
// themselves, so the box turns transparent and their renderer shows
// through it; Annotate draws on glass laid over it, and a recording covers
// the box. Over a tool, the page moves the system's pointer as the scene
// plays, as the system draws it over the real tools. Switched off, or
// outside the app, the box shows the desktop alone, drained of colour when
// off. A tool's desktop is wider than its box and centred in it, so it is
// never repainted as the window resizes; a recording keeps the desktop's
// proportions, as tall as the box's parent lets it be.

export function SettingsPreview({
  className,
  isOn = true,
  scene,
}: {
  scene: SettingsPreviewScene;
  className?: string;
  /** The setting's feature is on; off, the picture is greyed and still. */
  isOn?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const cursorRef = useRef<HTMLImageElement>(null);
  const wallpaper = useDesktopWallpaper();
  const { island: notch } = useDisplayNotches();
  const cursor = useSystemCursor();
  const definition = sceneOf(scene);
  const size = pictureSize(scene);
  const isTool = definition.kind === "tool";
  const values = JSON.stringify(scene);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    void paintStill(canvas, JSON.parse(values) as SettingsPreviewScene, {
      notch,
      wallpaper,
    });
  }, [notch, values, wallpaper]);

  const isNative = useSettingsPreview({
    cursorRef,
    desktop: { notch, wallpaper },
    isOn,
    ref,
    scene,
  });
  // Only the Ruler and OCR draw under the page, through a transparent box.
  const showsThrough =
    isNative && isOn && definition.kind === "tool" && !definition.glass;
  // A tool's picture, centred however much wider than the box it is; the
  // box is left to right, so the start is the left.
  const centred = {
    height: size.height,
    insetInlineStart: "50%",
    transform: "translateX(-50%)",
    width: size.width,
  };

  return (
    <div
      aria-hidden
      className={cn(
        "relative shrink-0 overflow-hidden rounded-control",
        !showsThrough && "bg-recess",
        className,
      )}
      dir="ltr"
      inert
      ref={ref}
      style={
        isTool
          ? { height: size.height }
          : { aspectRatio: `${String(size.width)} / ${String(size.height)}` }
      }
    >
      <canvas
        className={cn(
          "absolute max-w-none transition-[filter,opacity]",
          isTool ? "top-0" : "inset-0 size-full",
          showsThrough && "opacity-0",
          !isOn && "opacity-50 grayscale",
        )}
        ref={canvasRef}
        style={isTool ? centred : undefined}
      />
      {isTool ? (
        <div className="absolute top-0" style={centred}>
          <img
            alt=""
            className="invisible absolute start-0 top-0 max-w-none"
            ref={cursorRef}
            src={cursor.src}
            style={{
              height: cursor.height,
              marginInlineStart: -cursor.hotspotX,
              marginTop: -cursor.hotspotY,
              width: cursor.width,
            }}
          />
        </div>
      ) : null}
    </div>
  );
}
