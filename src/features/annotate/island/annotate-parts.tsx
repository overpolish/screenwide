// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import type { ComponentType } from "react";

import { Eraser, Undo2, X } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import {
  IconButton,
  IconToggleButton,
} from "../../../primitives/button/icon-button";
import { IslandParts } from "../../../primitives/island/island-parts";
import { swatchEdge } from "../../../primitives/swatch/swatch-styles";
import { annotationToolName } from "../../annotations/annotation-text";

import { AnnotateColors } from "./annotate-colors";
import { ISLAND_TOOLS } from "./island-tools";

import type { SectionProps } from "./section-props";
import type { AnnotatePartsProps } from "./use-annotate-parts";
import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

// The parts the annotate tools are made of, laid out by each island in its
// own way: the tools, what can be done besides drawing, and what hangs under
// them. The annotate island splits the first two round the notch; the
// recording island lines them up under its own row.

/** The tools, the one in hand chosen. Picking one steps back to its pen. */
export function AnnotateToolButtons({
  onChange,
  onPick,
  settings,
}: {
  onChange: AnnotatePartsProps["onChange"];
  onPick: () => void;
  settings: AnnotateSettings;
}) {
  return ISLAND_TOOLS.map(({ icon: Icon, id, name }) => (
    <IconToggleButton
      aria-label={name?.() ?? annotationToolName(id)}
      isSelected={id === settings.defaultShape}
      key={id}
      onChange={() => {
        if (id !== settings.defaultShape) onChange({ defaultShape: id });
        onPick();
      }}
    >
      <Icon aria-hidden className="size-icon" />
    </IconToggleButton>
  ));
}

/** The colour in hand, which opens the colours, then undo, clear and close.
 * Undo and clear are done with the colours, so they step back to the pen.
 * A tool that draws in no colour takes the colour away, as the island's
 * parts come and go (`IslandParts`). */
export function AnnotateActions({
  hasColor,
  onClear,
  onDone,
  onShowPen,
  onToggleColors,
  onUndo,
  settings,
  showsColors,
}: Pick<AnnotatePartsProps, "onClear" | "onDone" | "onUndo" | "settings"> & {
  hasColor: boolean;
  onShowPen: () => void;
  onToggleColors: () => void;
  showsColors: boolean;
}) {
  return (
    <IslandParts className="flex items-center gap-separate" direction="row">
      {hasColor ? (
        <IconButton
          aria-expanded={showsColors}
          aria-label={t("annotation-color")}
          key="color"
          onPress={onToggleColors}
          variant="ghost"
        >
          <span
            className={cn("size-icon rounded-full", swatchEdge)}
            style={{ backgroundColor: settings.defaultColor }}
          />
        </IconButton>
      ) : null}
      <IconButton
        aria-label={t("annotate-undo-label")}
        key="undo"
        onPress={() => {
          onShowPen();
          onUndo();
        }}
        variant="ghost"
      >
        <Undo2 aria-hidden className="size-icon rtl:-scale-x-100" />
      </IconButton>
      <IconButton
        aria-label={t("annotate-clear-label")}
        key="clear"
        onPress={() => {
          onShowPen();
          onClear();
        }}
        variant="ghost"
      >
        <Eraser aria-hidden className="size-icon" />
      </IconButton>
      <IconButton
        aria-label={t("annotate-close-label")}
        key="close"
        onPress={onDone}
        variant="ghost"
      >
        <X aria-hidden className="size-icon" />
      </IconButton>
    </IslandParts>
  );
}

/** What hangs under the tools: the colours, or the pen of the tool in hand,
 * one giving way to the other as the island's parts do (`IslandParts`). */
export function AnnotateOptions({
  onChange,
  pen: Pen,
  savedColors,
  settings,
  showsColors,
}: Pick<AnnotatePartsProps, "onChange" | "savedColors" | "settings"> & {
  pen: ComponentType<SectionProps> | undefined;
  showsColors: boolean;
}) {
  return (
    <IslandParts>
      {showsColors ? (
        <AnnotateColors
          key="colors"
          onChange={onChange}
          savedColors={savedColors}
          settings={settings}
        />
      ) : Pen ? (
        <Pen
          key={settings.defaultShape}
          onChange={onChange}
          settings={settings}
        />
      ) : null}
    </IslandParts>
  );
}
