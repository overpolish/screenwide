// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ReactNode } from "react";

import { Toolbar } from "react-aria-components";

import { t } from "../../../i18n/i18n";

import {
  AnnotateActions,
  AnnotateOptions,
  AnnotateToolButtons,
} from "./annotate-parts";
import {
  type AnnotatePartsProps,
  useAnnotateParts,
} from "./use-annotate-parts";

import type { IslandAnchor } from "../../../primitives/island/island-geometry";

/** The annotate tools laid out for another island, as the recording island
 * shows them while it records. */
export type AnnotateRows = {
  /** Where under the tools what hangs from them sits: the pen under the
   * tools' start, the colours under the colour, at the end. */
  anchor: IslandAnchor;
  /** The pen of the tool in hand, or the colours, to hang under the tools. */
  options: ReactNode;
  /** The tools at the start of one line and what can be done besides
   * drawing at its end, with no notch to part them, to lay under the
   * island's row in its black. A toolbar of its own, so arrow keys move
   * along it as they do along the row. */
  tools: ReactNode;
};

/** The annotate tools as a line under another island's row and what hangs
 * from it, sharing one state, so the island can lay them out as the
 * annotate island does: the line in the row's black, the options hanging
 * from it. `null` while there are no tools to show. */
export function useAnnotateRows(
  props: AnnotatePartsProps | null,
): AnnotateRows | null {
  const { hasColor, pen, showPen, showsColors, toggleColors } =
    useAnnotateParts({
      initialSection: props?.initialSection,
      shape: props?.settings.defaultShape,
    });
  if (!props) return null;
  const { onChange, onClear, onDone, onUndo, savedColors, settings } = props;
  return {
    anchor: showsColors ? "end" : "start",
    options: (
      <AnnotateOptions
        onChange={onChange}
        pen={pen}
        savedColors={savedColors}
        settings={settings}
        showsColors={showsColors}
      />
    ),
    tools: (
      <Toolbar
        aria-label={t("annotate-tools")}
        className="flex h-10 items-center justify-between gap-separate"
      >
        <span className="flex items-center gap-separate">
          <AnnotateToolButtons
            onChange={onChange}
            onPick={showPen}
            settings={settings}
          />
        </span>
        <span className="flex items-center gap-separate">
          <AnnotateActions
            hasColor={hasColor}
            onClear={onClear}
            onDone={onDone}
            onToggleColors={toggleColors}
            onUndo={onUndo}
            settings={settings}
            showsColors={showsColors}
          />
        </span>
      </Toolbar>
    ),
  };
}
