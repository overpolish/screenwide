// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ellipsis } from "lucide-react";
import { use, useEffect, useRef, useState } from "react";

import { t } from "../../../i18n/i18n";
import { IconButton } from "../../../primitives/button/icon-button";
import { Toolbar } from "../../../primitives/toolbar/toolbar";
import { ToolbarDisabledContext } from "../../../primitives/toolbar/toolbar-disabled";
import { ToolbarTool } from "../../../primitives/toolbar/toolbar-tool";
import { TooltipTrigger } from "../../../primitives/tooltip/tooltip-trigger";
import {
  annotationToolLabel,
  annotationToolName,
} from "../../annotations/annotation-text";
import { ANNOTATION_TOOLS, AnnotationTool } from "../tool-panels/tool-registry";

import { annotationToolLayout } from "./annotation-tool-layout";
import {
  closeAnnotationToolMenu,
  useAnnotationToolMenu,
} from "./use-annotation-tool-menu";

import type { AnnotationKind } from "../../annotations/types";

/**
 * The annotation tools. The tool in hand always has a button: one hidden in
 * the menu takes the slot before the menu button when it is taken up, by the
 * menu or by its shortcut.
 */
export function AnnotationToolStrip({
  isDisabled = false,
  onChoose,
  tool,
}: {
  onChoose: (id: AnnotationKind, selected: boolean) => void;
  tool: string | null;
  isDisabled?: boolean;
}) {
  const isBarDisabled = use(ToolbarDisabledContext);
  const moreRef = useRef<HTMLButtonElement>(null);
  const [slotted, setSlotted] = useState<AnnotationTool | null>(null);
  const openMenu = useAnnotationToolMenu(({ id }) => {
    onChoose(id, true);
  });
  // Taking up a tool any other way, by its shortcut or its button, answers
  // the menu as surely as picking from it would.
  useEffect(() => {
    void closeAnnotationToolMenu();
  }, [tool]);

  const layout = annotationToolLayout(ANNOTATION_TOOLS, null, slotted);
  const active = ANNOTATION_TOOLS.find(({ id }) => id === tool);
  if (active && layout.hidden.includes(active)) setSlotted(active);

  const shown = layout.slot ? [...layout.visible, layout.slot] : layout.visible;

  return (
    <div className="flex">
      <Toolbar aria-label={t("annotate-tools")}>
        {/* No `data-editor-tool` marker: the annotation panel hangs from the
          picture, never from a toolbar button. */}
        {shown.map(({ icon: Icon, id, shortcut }) => (
          <ToolbarTool
            aria-label={annotationToolName(id)}
            isDisabled={isDisabled}
            isSelected={tool === id && !isDisabled}
            key={id}
            label={annotationToolLabel(id)}
            onChange={(selected) => {
              onChoose(id, selected);
            }}
            shortcut={shortcut}
          >
            <Icon />
          </ToolbarTool>
        ))}
        {layout.hidden.length > 0 ? (
          <TooltipTrigger tooltip={t("editor-toolbar-more-tools")}>
            <IconButton
              aria-label={t("editor-toolbar-more-annotation-tools")}
              className="text-content-fg-secondary"
              isDisabled={isDisabled || isBarDisabled}
              onPress={() => {
                const bounds = moreRef.current?.getBoundingClientRect();
                if (bounds) void openMenu(layout.hidden, bounds);
              }}
              ref={moreRef}
            >
              <Ellipsis />
            </IconButton>
          </TooltipTrigger>
        ) : null}
      </Toolbar>
    </div>
  );
}
