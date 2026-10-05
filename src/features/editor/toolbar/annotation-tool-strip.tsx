// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ellipsis } from "lucide-react";
import { use, useEffect, useLayoutEffect, useRef, useState } from "react";

import { IconButton } from "../../../components/base/button/icon-button";
import { ButtonGroup } from "../../../components/base/button-group/button-group";
import { NativeTooltipTrigger } from "../../../components/shared/native-tooltip/native-tooltip-trigger";
import { ToolToggle } from "../../../components/shared/tool-toggle/tool-toggle";
import { ToolToggleDisabledContext } from "../../../components/shared/tool-toggle/tool-toggle-disabled";
import { WindowHeaderCenterWidthContext } from "../../../components/shared/window-header/window-header-center-width";
import { ANNOTATION_TOOLS, AnnotationTool } from "../tool-panels/tool-registry";

import { annotationToolLayout } from "./annotation-tool-layout";
import {
  closeAnnotationToolMenu,
  useAnnotationToolMenu,
} from "./use-annotation-tool-menu";

import type { AnnotationKind } from "../../../components/shared/annotation-style/types";

/**
 * How many equal buttons the strip has room for: the title bar's centre less
 * the groups beside the strip, which do not depend on it, so a measurement
 * never feeds its own result.
 */
function measureCapacity(strip: HTMLElement, centerWidth: number) {
  const row = strip.parentElement;
  const group = strip.firstElementChild;
  const button = strip.querySelector("button");
  if (!row || !group || !button) return null;
  const beside =
    row.getBoundingClientRect().width - strip.getBoundingClientRect().width;
  const gap = parseFloat(getComputedStyle(group).columnGap) || 0;
  const step = button.getBoundingClientRect().width + gap;
  // Half a pixel of slack, so subpixel rounding does not cost a button.
  return Math.floor((centerWidth - beside + gap + 0.5) / step);
}

/**
 * The annotation tools in the title bar, folding the least used into a menu
 * when the bar is too narrow for all of them. The tool in hand always has a
 * button: one hidden in the menu takes the slot before the menu button when
 * it is taken up, by the menu or by its shortcut.
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
  const centerWidth = use(WindowHeaderCenterWidthContext);
  const isBarDisabled = use(ToolToggleDisabledContext);
  const stripRef = useRef<HTMLDivElement>(null);
  const moreRef = useRef<HTMLButtonElement>(null);
  const [capacity, setCapacity] = useState<number | null>(null);
  const [slotted, setSlotted] = useState<AnnotationTool | null>(null);
  const openMenu = useAnnotationToolMenu(({ id }) => {
    onChoose(id, true);
  });
  // Taking up a tool any other way, by its shortcut or its button, answers
  // the menu as surely as picking from it would.
  useEffect(() => {
    void closeAnnotationToolMenu();
  }, [tool]);

  useLayoutEffect(() => {
    const strip = stripRef.current;
    const row = strip?.parentElement;
    if (centerWidth === null || !strip || !row) return;
    // The observer's first report, before paint, is the first measurement;
    // later ones follow the groups beside the strip, which come and go with
    // what a recording holds.
    const observer = new ResizeObserver(() => {
      setCapacity(measureCapacity(strip, centerWidth));
    });
    observer.observe(row);
    return () => {
      observer.disconnect();
    };
  }, [centerWidth]);

  const layout = annotationToolLayout(
    ANNOTATION_TOOLS,
    centerWidth === null ? null : capacity,
    slotted,
  );
  const active = ANNOTATION_TOOLS.find(({ id }) => id === tool);
  if (active && layout.hidden.includes(active)) setSlotted(active);

  const shown = layout.slot ? [...layout.visible, layout.slot] : layout.visible;

  return (
    <div className="flex" ref={stripRef}>
      <ButtonGroup aria-label="Annotation tools" className="gap-control">
        {/* No `data-editor-tool` marker: the annotation panel hangs from the
            picture, never from a toolbar button. */}
        {shown.map(({ icon: Icon, id, label, name, shortcut }) => (
          <ToolToggle
            isDisabled={isDisabled}
            isSelected={tool === id && !isDisabled}
            key={id}
            label={label}
            name={name}
            onSelectedChange={(selected) => {
              onChoose(id, selected);
            }}
            shortcut={shortcut}
          >
            <Icon />
          </ToolToggle>
        ))}
        {layout.hidden.length > 0 ? (
          <NativeTooltipTrigger tooltip="More tools">
            <IconButton
              aria-label="More annotation tools"
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
          </NativeTooltipTrigger>
        ) : null}
      </ButtonGroup>
    </div>
  );
}
