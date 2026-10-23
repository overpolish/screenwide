// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useState } from "react";

import { ISLAND_TOOLS } from "./island-tools";

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";
import type { AnnotationKind } from "../../../bindings/AnnotationKind";

/** What hangs under the tools: the pen of the tool in hand, or the colours. */
export type AnnotateSection = "colors" | "pen";

/** What the annotate tools take wherever an island shows them. */
export type AnnotatePartsProps = {
  /** Every change is a settings edit: the tools and the Settings page are one
   * state, so what is chosen here dresses the next stroke and shows up there
   * without either being reloaded.
   *
   * `immediate` is for an edit that has to reach the overlay before the next
   * stroke rather than when the hand settles, where there is no drag to wait
   * for. */
  onChange: (patch: Partial<AnnotateSettings>, immediate?: boolean) => void;
  onClear: () => void;
  /** Closes the overlay, the way Escape does. */
  onDone: () => void;
  onUndo: () => void;
  settings: AnnotateSettings;
  /** What hangs under the tools when they first show. */
  initialSection?: AnnotateSection;
  /** The colours of your own, as the editor kept them. Shown after the
   * palette. */
  savedColors?: string[];
};

// The annotate tools' one piece of state, kept the same wherever an island
// shows them: whether the pen or the colours hang under the tools. The
// colours stay open while colours are tried; pressing the colour again, a
// tool or Escape steps back to the pen. A tool that draws in no colour hangs
// its own pen, even where the colours were open for the tool before it.

export function useAnnotateParts({
  initialSection = "pen",
  shape,
}: {
  /** The tool in hand; none while the settings are still being read. */
  shape: AnnotationKind | undefined;
  initialSection?: AnnotateSection;
}) {
  const [section, setSection] = useState<AnnotateSection>(initialSection);
  const tool = ISLAND_TOOLS.find(({ id }) => id === shape);
  const hasColor = tool?.hasNoColor !== true;

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setSection("pen");
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  const showPen = useCallback(() => {
    setSection("pen");
  }, []);
  const toggleColors = useCallback(() => {
    setSection((current) => (current === "colors" ? "pen" : "colors"));
  }, []);

  return {
    hasColor,
    pen: tool?.pen,
    showPen,
    showsColors: section === "colors" && hasColor,
    toggleColors,
  };
}
