// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop, MousePointer2, SquareDashedMousePointer } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { Toolbar } from "../../../../primitives/toolbar/toolbar";
import { ToolbarTool } from "../../../../primitives/toolbar/toolbar-tool";
import { FrameToolIcon } from "../../glyphs/frame-tool-icon";
import { SceneToolToggle } from "../../tool-panels/scene/scene-tool-toggle";
import { AnnotationToolStrip } from "../../toolbar/annotation-tool-strip";

import type { AnnotationKind } from "../../../annotations/types";

export type RecordingCanvasTool =
  AnnotationKind | "canvas" | "crop" | "marquee" | "scene" | "select" | null;

/** The recording's pointer, canvas and annotation groups. */
export function RecordingCanvasTools({
  hasAnnotations,
  hasScenes,
  isEnabled,
  isArrowEnabled = isEnabled,
  isFrameEnabled = isEnabled,
  isSelectEnabled = isEnabled,
  onToolChange,
  tool,
}: {
  /** Whether annotations can be drawn, which the annotation tools are for. */
  hasAnnotations: boolean;
  /** Whether the recording has a screen and a camera to arrange. */
  hasScenes: boolean;
  isEnabled: boolean;
  onToolChange: (tool: RecordingCanvasTool) => void;
  tool: RecordingCanvasTool;
  isArrowEnabled?: boolean;
  isFrameEnabled?: boolean;
  isSelectEnabled?: boolean;
}) {
  return (
    <>
      <Toolbar aria-label={t("editor-pointer-tools")}>
        {/* The marker is the anchor Select's panel hangs from: taking the tool
          up opens it, however the tool was taken up. */}
        <span className="inline-flex" data-editor-tool="select">
          <ToolbarTool
            aria-label={t("editor-tool-select-clip-name")}
            isDisabled={!isSelectEnabled}
            isSelected={tool === "select" && isSelectEnabled}
            label={t("editor-tool-select")}
            onChange={(selected) => {
              onToolChange(selected ? "select" : null);
            }}
            shortcut="V"
          >
            <MousePointer2 />
          </ToolbarTool>
        </span>
        <ToolbarTool
          aria-label={t("editor-tool-marquee-name")}
          isDisabled={!isArrowEnabled}
          isSelected={tool === "marquee" && isArrowEnabled}
          label={t("editor-tool-marquee")}
          onChange={(selected) => {
            onToolChange(selected ? "marquee" : null);
          }}
          shortcut="G"
        >
          <SquareDashedMousePointer />
        </ToolbarTool>
      </Toolbar>
      <Toolbar aria-label={t("editor-canvas-tools")}>
        <ToolbarTool
          aria-label={t("editor-tool-frame-name")}
          isDisabled={!isFrameEnabled}
          isSelected={tool === "canvas" && isFrameEnabled}
          label={t("editor-tool-frame")}
          onChange={(selected) => {
            onToolChange(selected ? "canvas" : null);
          }}
          shortcut="F"
        >
          <FrameToolIcon />
        </ToolbarTool>
        {hasScenes ? (
          <SceneToolToggle
            isSelected={tool === "scene"}
            onSelectedChange={(selected) => {
              onToolChange(selected ? "scene" : null);
            }}
          />
        ) : null}
        <ToolbarTool
          aria-label={t("editor-tool-crop-clip-name")}
          isDisabled={!isEnabled}
          isSelected={tool === "crop" && isEnabled}
          label={t("editor-tool-crop")}
          onChange={(selected) => {
            onToolChange(selected ? "crop" : null);
          }}
          shortcut="C"
        >
          <Crop />
        </ToolbarTool>
      </Toolbar>
      {hasAnnotations ? (
        <AnnotationToolStrip
          isDisabled={!isArrowEnabled}
          onChoose={(id, selected) => {
            onToolChange(selected ? id : null);
          }}
          tool={tool}
        />
      ) : null}
    </>
  );
}
