// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Eraser, GripVertical, Undo2, X } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";

import { IconButton } from "../../components/base/button/icon-button";
import { ButtonGroup } from "../../components/base/button-group/button-group";
import { AnnotationAngleDial } from "../../components/shared/annotation-style/annotation-angle-dial";
import { AnnotationColorGrid } from "../../components/shared/annotation-style/annotation-color-grid";
import { AnnotationHeadGroup } from "../../components/shared/annotation-style/annotation-head-group";
import { AnnotationWidthSlider } from "../../components/shared/annotation-style/annotation-width-slider";
import { annotationSizes } from "../../components/shared/annotation-style/widths";
import { BackgroundTile } from "../../components/shared/background-picker/background-tile";
import { NativeTooltipTrigger } from "../../components/shared/native-tooltip/native-tooltip-trigger";
import { ToolToggle } from "../../components/shared/tool-toggle/tool-toggle";
import { AnnotateSettings } from "../settings/types";

import { ANNOTATE_TOOLS } from "./annotate-tools";

export type AnnotateToolbarProps = {
  /** Every change is a settings edit: the toolbar and the Settings page are
   * one state, so what is chosen here dresses the next stroke and shows up
   * there without either being reloaded.
   *
   * `immediate` is for an edit that has to reach the overlay before the next
   * stroke rather than when the hand settles: a typed value, where there is no
   * drag to wait for. */
  onChange: (patch: Partial<AnnotateSettings>, immediate?: boolean) => void;
  onClear: () => void;
  /** Closes the overlay, the way Escape does: the plate's one way out for
   * a pointer. */
  onDone: () => void;
  onUndo: () => void;
  settings: AnnotateSettings;
  /** The plate reporting what it measures, in logical px. The window is sized
   * to it. */
  onSizeChange?: (width: number, height: number) => void;
  /** The colours of your own, as the editor's arrow panel kept them. Shown
   * after the palette; they are added and forgotten in the editor, which is
   * where the system Colours panel can be reached. */
  savedColors?: string[];
};

/**
 * The live overlay's toolbar: what the next stroke will be, and the two things
 * that can be done to what is already drawn.
 *
 * Live mode is draw-only - nothing on screen can be picked up or moved - so
 * the plate carries no selection controls: undo and clear are the only ways an
 * annotation changes. Close leaves the overlay, as Escape does.
 *
 * The colours open as a second row of the plate rather than in a layer of
 * their own: the window is the plate, so a popover would be clipped by it.
 * The window grows to hold the row and shrinks again when it closes.
 */
export function AnnotateToolbar({
  onChange,
  onClear,
  onDone,
  onSizeChange,
  onUndo,
  savedColors,
  settings,
}: AnnotateToolbarProps) {
  const [showColors, setShowColors] = useState(false);
  const plateRef = useRef<HTMLElement>(null);
  // Which controls the plate carries: a counter has a disc and an aim where
  // an arrow has a stroke and a head.
  const isCounter = settings.defaultShape === "counter";

  useLayoutEffect(() => {
    const plate = plateRef.current;
    if (!plate || !onSizeChange) return;

    const report = () => {
      const rect = plate.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return;
      onSizeChange(Math.ceil(rect.width), Math.ceil(rect.height));
    };
    const observer = new ResizeObserver(report);
    observer.observe(plate);
    report();

    return () => {
      observer.disconnect();
    };
  }, [onSizeChange]);

  return (
    <main
      className="window-surface flex w-max flex-col gap-control-inset rounded-window p-control-inset text-content-fg windows:inset-ring windows:inset-ring-popover-stroke"
      data-tauri-drag-region="deep"
      ref={plateRef}
    >
      <div className="flex items-center gap-section">
        {/* The grip is the plate's own handle: the whole plate drags, and this
            is what says so. */}
        <GripVertical
          aria-hidden="true"
          className="size-icon shrink-0 text-content-fg-tertiary"
        />

        {/* The tools, as the Editor's own toolbars show them: one cluster of
            toggles, each naming itself in a tooltip. */}
        <ButtonGroup aria-label="Annotation tools" className="gap-control">
          {ANNOTATE_TOOLS.map((tool) => (
            <ToolToggle
              isSelected={tool.id === settings.defaultShape}
              key={tool.id}
              label={tool.label}
              name={tool.name}
              onSelectedChange={(selected) => {
                // The overlay always has a tool in hand, so pressing the one
                // already chosen leaves it chosen.
                if (selected) onChange({ defaultShape: tool.id });
              }}
              shortcut={tool.shortcut}
            >
              {tool.icon}
            </ToolToggle>
          ))}
        </ButtonGroup>

        <div className="flex items-center gap-control-inset">
          <BackgroundTile
            ariaLabel="Colour"
            background={{ color: settings.defaultColor, kind: "solid" }}
            id="color"
            isSelected={showColors}
            onPress={() => {
              setShowColors((open) => !open);
            }}
          />
          <AnnotationWidthSlider
            label={isCounter ? "Size" : "Width"}
            onChange={(size) => {
              onChange(
                isCounter
                  ? { defaultCounterSize: size }
                  : { defaultWidth: size },
              );
            }}
            presets={annotationSizes(settings.defaultShape)}
            value={
              isCounter ? settings.defaultCounterSize : settings.defaultWidth
            }
          />
          {/* The head belongs to the arrow and the aim to the counter, so the
              plate carries whichever the tool in hand has. A live annotation cannot
              be picked up again, so a counter is aimed before it is dropped
              rather than turned afterwards. */}
          {isCounter ? (
            <AnnotationAngleDial
              onChange={(defaultCounterAngle, typed) => {
                onChange({ defaultCounterAngle }, typed);
              }}
              value={settings.defaultCounterAngle}
            />
          ) : (
            <AnnotationHeadGroup
              onChange={(defaultHead) => {
                onChange({ defaultHead });
              }}
              value={settings.defaultHead}
            />
          )}
        </div>

        <div className="flex items-center gap-control">
          <NativeTooltipTrigger tooltip={{ label: "Undo", shortcut: "⌘Z" }}>
            <IconButton aria-label="Undo the last annotation" onPress={onUndo}>
              <Undo2 />
            </IconButton>
          </NativeTooltipTrigger>
          <NativeTooltipTrigger tooltip={{ label: "Clear", shortcut: "⌫" }}>
            <IconButton aria-label="Clear annotations" onPress={onClear}>
              <Eraser />
            </IconButton>
          </NativeTooltipTrigger>
          <NativeTooltipTrigger tooltip={{ label: "Close", shortcut: "Esc" }}>
            <IconButton
              aria-label="Close the annotate overlay"
              onPress={onDone}
            >
              <X />
            </IconButton>
          </NativeTooltipTrigger>
        </div>
      </div>

      {showColors ? (
        <AnnotationColorGrid
          onChange={(defaultColor) => {
            onChange({ defaultColor });
            setShowColors(false);
          }}
          savedColors={savedColors}
          value={settings.defaultColor}
        />
      ) : null}
    </main>
  );
}
