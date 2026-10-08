// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Eraser, GripVertical, Undo2, X } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";

import { IconButton } from "../../components/base/button/icon-button";
import { ButtonGroup } from "../../components/base/button-group/button-group";
import { AnnotationColorGrid } from "../../components/shared/annotation-style/annotation-color-grid";
import {
  annotationToolLabel,
  annotationToolName,
} from "../../components/shared/annotation-style/annotation-text";
import { BackgroundTile } from "../../components/shared/background-picker/background-tile";
import { NativeTooltipTrigger } from "../../components/shared/native-tooltip/native-tooltip-trigger";
import { ToolToggle } from "../../components/shared/tool-toggle/tool-toggle";
import { t } from "../../i18n/i18n";

import { AnnotateToolControls } from "./annotate-tool-controls";
import { ANNOTATE_TOOLS } from "./annotate-tools";

import type { AnnotateSettings } from "../../bindings/AnnotateSettings";

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
  // A spotlight is no colour: its shade is the same black whatever the
  // arrows are drawn in, so the plate offers no colour while it is in hand.
  const hasColor = settings.defaultShape !== "spotlight";

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

  // A tool change that lands while the toolbar is hidden - a write from the
  // Settings page between sessions - is measured as it lands: a hidden
  // window's resize observer can wait for a frame it is not drawing, and the
  // next session would open the plate at the size of the tool it had before.
  useLayoutEffect(() => {
    const rect = plateRef.current?.getBoundingClientRect();
    if (!rect || !onSizeChange || rect.width === 0 || rect.height === 0) return;
    onSizeChange(Math.ceil(rect.width), Math.ceil(rect.height));
  }, [onSizeChange, settings.defaultShape]);

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
        <ButtonGroup aria-label={t("annotate-tools")} className="gap-control">
          {ANNOTATE_TOOLS.map((tool) => (
            <ToolToggle
              isSelected={tool.id === settings.defaultShape}
              key={tool.id}
              label={annotationToolLabel(tool.id)}
              name={annotationToolName(tool.id)}
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
          {hasColor ? (
            <BackgroundTile
              ariaLabel={t("annotation-color")}
              background={{ color: settings.defaultColor, kind: "solid" }}
              id="color"
              isSelected={showColors}
              onPress={() => {
                setShowColors((open) => !open);
              }}
            />
          ) : null}
          <AnnotateToolControls onChange={onChange} settings={settings} />
        </div>

        <div className="flex items-center gap-control">
          <NativeTooltipTrigger
            tooltip={{
              label: t("annotate-undo"),
              shortcut: "CommandOrControl+KeyZ",
            }}
          >
            <IconButton aria-label={t("annotate-undo-label")} onPress={onUndo}>
              <Undo2 className="rtl:-scale-x-100" />
            </IconButton>
          </NativeTooltipTrigger>
          <NativeTooltipTrigger
            tooltip={{ label: t("annotate-clear"), shortcut: "Backspace" }}
          >
            <IconButton
              aria-label={t("annotate-clear-label")}
              onPress={onClear}
            >
              <Eraser />
            </IconButton>
          </NativeTooltipTrigger>
          <NativeTooltipTrigger
            tooltip={{ label: t("annotate-close"), shortcut: "Esc" }}
          >
            <IconButton aria-label={t("annotate-close-label")} onPress={onDone}>
              <X />
            </IconButton>
          </NativeTooltipTrigger>
        </div>
      </div>

      {showColors && hasColor ? (
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
