// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { SceneLayoutGlyph } from "../../glyphs/scene-layout-glyph";
import { reach, type Span, type TimelineScale } from "../timeline-scale";

import type { SceneDrawing } from "../../glyphs/scene-layout-drawing";

// One scene over the stretch it frames. What it shows grows with its width,
// as a beat's does: bare, then its layout, then its layout and name, cut
// short where it runs out of room. Chosen, it fills and grows a handle at
// each end. A scene a few pixels wide is pressed across more than it is
// drawn.

/** The narrowest a scene shows its layout at, in px: the glyph's widest
 * and the padding either side. */
const GLYPH_PX = 30;
/** The narrowest a scene shows its name beside its layout at, in px. */
const NAME_PX = 72;

export type TimelineScene = Span & {
  drawing: SceneDrawing;
  id: string;
  /** What its layout is called. */
  name: string;
};

export type SceneClipProps = {
  isSelected: boolean;
  /** Told a press on one of a chosen scene's ends, which takes hold of it;
   * the moves and release that follow bubble up from it. */
  onGrabEdge: (event: React.PointerEvent, edge: "end" | "start") => void;
  onPress: () => void;
  scale: TimelineScale;
  scene: TimelineScene;
  /** The strip's width, in px. */
  width: number;
};

export function SceneClip({
  isSelected,
  onGrabEdge,
  onPress,
  scale,
  scene,
  width,
}: SceneClipProps) {
  const { pad, px, style } = reach(scale, scene, width);
  return (
    <button
      aria-label={t("editor-timeline-scene", { layout: scene.name })}
      aria-pressed={isSelected}
      className="group absolute inset-y-0 cursor-default outline-none"
      dir="ltr"
      onClick={onPress}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={style}
      type="button"
    >
      <span
        className={cn(
          "absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-part text-caption whitespace-nowrap transition-colors group-focus-visible:outline-2 group-focus-visible:outline-focus",
          // Padding would hold a narrow scene wider than its time.
          px >= GLYPH_PX && "px-1",
          isSelected
            ? "bg-primary text-on-primary"
            : "bg-fill text-fg-secondary group-hover:bg-fill-strong",
        )}
        style={{ insetInlineEnd: pad, insetInlineStart: pad }}
      >
        {px >= GLYPH_PX ? (
          <SceneLayoutGlyph drawing={scene.drawing} isFramed={false} />
        ) : null}
        {px >= NAME_PX ? <span className="truncate">{scene.name}</span> : null}
      </span>
      {isSelected
        ? (["start", "end"] as const).map((edge) => (
            <span
              aria-hidden
              className="absolute inset-y-0 flex w-3 -translate-x-1/2 cursor-ew-resize items-center justify-center"
              key={edge}
              onPointerDown={(event) => {
                event.stopPropagation();
                event.currentTarget.setPointerCapture(event.pointerId);
                onGrabEdge(event, edge);
              }}
              style={{
                insetInlineStart:
                  edge === "start" ? pad : `calc(100% - ${String(pad)}px)`,
              }}
            >
              <span className="h-3.5 w-1 rounded-full bg-on-primary ring-2 ring-primary" />
            </span>
          ))
        : null}
    </button>
  );
}
