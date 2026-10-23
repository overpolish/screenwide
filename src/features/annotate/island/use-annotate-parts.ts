// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useState } from "react";

import { ISLAND_TOOLS } from "./island-tools";

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";
import type { AnnotateToolbarProps } from "../annotate-toolbar";

/** What hangs under the tools: the pen of the tool in hand, or the colours. */
export type AnnotateSection = "colors" | "pen";

/** What the annotate tools take wherever an island shows them. */
export type AnnotatePartsProps = Omit<AnnotateToolbarProps, "onSizeChange"> & {
  /** What hangs under the tools when they first show. */
  initialSection?: AnnotateSection;
};

// The annotate tools' one piece of state, kept the same wherever an island
// shows them: whether the pen or the colours hang under the tools. The
// colours stay open while colours are tried; pressing the colour again, a
// tool or Escape steps back to the pen. A tool that draws in no colour hangs
// its own pen, even where the colours were open for the tool before it.

export function useAnnotateParts({
  initialSection = "pen",
  settings,
}: {
  settings: AnnotateSettings;
  initialSection?: AnnotateSection;
}) {
  const [section, setSection] = useState<AnnotateSection>(initialSection);
  const tool = ISLAND_TOOLS.find(({ id }) => id === settings.defaultShape);
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
