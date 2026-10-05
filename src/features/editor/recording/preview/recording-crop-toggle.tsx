// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Crop,
  MousePointer2,
  ScanSquare,
  SquareDashedMousePointer,
} from "lucide-react";

import { ButtonGroup } from "../../../../components/base/button-group/button-group";
import { ToolToggle } from "../../../../components/shared/tool-toggle/tool-toggle";
import { SceneToolToggle } from "../../tool-panels/scene/scene-tool-toggle";
import { AnnotationToolStrip } from "../../toolbar/annotation-tool-strip";

import type { AnnotationKind } from "../../../../components/shared/annotation-style/types";

export type RecordingCanvasTool =
  AnnotationKind | "canvas" | "crop" | "marquee" | "scene" | "select" | null;

/** The recording's pointer, canvas and annotation groups. */
export function RecordingCanvasTools({
  hasScenes,
  isEnabled,
  isArrowEnabled = isEnabled,
  isFrameEnabled = isEnabled,
  isSelectEnabled = isEnabled,
  onToolChange,
  tool,
}: {
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
      <ButtonGroup aria-label="Pointer tools" className="gap-control">
        {/* The marker is the anchor Select's panel hangs from: taking the tool
            up opens it, however the tool was taken up. */}
        <span className="inline-flex" data-editor-tool="select">
          <ToolToggle
            isDisabled={!isSelectEnabled}
            isSelected={tool === "select" && isSelectEnabled}
            label="Select"
            name="Select recording clip"
            onSelectedChange={(selected) => {
              onToolChange(selected ? "select" : null);
            }}
            shortcut="V"
          >
            <MousePointer2 />
          </ToolToggle>
        </span>
        <ToolToggle
          isDisabled={!isArrowEnabled}
          isSelected={tool === "marquee" && isArrowEnabled}
          label="Marquee"
          name="Choose annotations with a marquee"
          onSelectedChange={(selected) => {
            onToolChange(selected ? "marquee" : null);
          }}
          shortcut="G"
        >
          <SquareDashedMousePointer />
        </ToolToggle>
      </ButtonGroup>
      <ButtonGroup aria-label="Canvas tools" className="gap-control">
        <ToolToggle
          isDisabled={!isFrameEnabled}
          isSelected={tool === "canvas" && isFrameEnabled}
          label="Frame"
          name="Resize recording frame"
          onSelectedChange={(selected) => {
            onToolChange(selected ? "canvas" : null);
          }}
          shortcut="F"
        >
          <ScanSquare />
        </ToolToggle>
        {hasScenes ? (
          <SceneToolToggle
            isSelected={tool === "scene"}
            onSelectedChange={(selected) => {
              onToolChange(selected ? "scene" : null);
            }}
          />
        ) : null}
        <ToolToggle
          isDisabled={!isEnabled}
          isSelected={tool === "crop" && isEnabled}
          label="Crop"
          name="Crop recording clip"
          onSelectedChange={(selected) => {
            onToolChange(selected ? "crop" : null);
          }}
          shortcut="C"
        >
          <Crop />
        </ToolToggle>
      </ButtonGroup>
      <AnnotationToolStrip
        isDisabled={!isArrowEnabled}
        onChoose={(id, selected) => {
          onToolChange(selected ? id : null);
        }}
        tool={tool}
      />
    </>
  );
}
