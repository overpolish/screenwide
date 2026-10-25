// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { SliderNumberField } from "../../../../composites/slider-number-field/slider-number-field";
import { SettingsGroup } from "../../../../features/settings/settings-group";
import { isMacHost } from "../../../../lib/platform";
import { Switch } from "../../../../primitives/switch/switch";
import { GlideArt } from "../preview/glide-art";
import { AnnotateArt, OcrArt, RulerArt } from "../preview/tool-art";

import { Pane } from "./pane";

import type { ExploredSettings } from "../settings-state";

// Tools: each tool in a box of its own, opening on the tool at work, then
// its switch and how it behaves. The picture follows the rows: the gap
// slider moves Glide's windows apart, Keep annotations leaves the arrow on
// screen, and switching a tool off takes its work out of the picture.

function SwitchRow({
  description,
  isDisabled,
  isSelected,
  onChange,
  title,
}: {
  isSelected: boolean;
  onChange: (isSelected: boolean) => void;
  title: string;
  description?: string;
  isDisabled?: boolean;
}) {
  return (
    <ControlRow description={description} title={title}>
      {(controlProps) => (
        <Switch
          {...controlProps}
          isDisabled={isDisabled}
          isSelected={isSelected}
          onChange={onChange}
        />
      )}
    </ControlRow>
  );
}

function ToolGroup({
  art,
  children,
  title,
}: {
  art: ReactNode;
  children: ReactNode;
  title: string;
}) {
  return (
    <SettingsGroup title={title}>
      {art}
      {children}
    </SettingsGroup>
  );
}

export function ToolsPane({ settings }: { settings: ExploredSettings }) {
  const {
    annotate,
    glide,
    ocr,
    ruler,
    setAnnotate,
    setGlide,
    setOcr,
    setRuler,
  } = settings;
  return (
    <Pane label="Tools">
      {glide ? (
        <ToolGroup
          art={
            <GlideArt
              cursorFollows={glide.cursorFollows}
              gap={glide.windowGap}
              isOn={glide.enabled}
            />
          }
          title="Glide"
        >
          <SwitchRow
            description="Drag a window to an edge and it fills that part of the screen."
            isSelected={glide.enabled}
            onChange={(enabled) => {
              setGlide({ ...glide, enabled });
            }}
            title="Move and resize windows to fit your screen"
          />
          <ControlRow title="Space around windows">
            {(controlProps) => (
              <div {...controlProps} role="group">
                <SliderNumberField
                  aria-label="Space around windows"
                  className="w-56"
                  defaultValue={0}
                  isDisabled={!glide.enabled}
                  maxValue={32}
                  minValue={0}
                  onChange={(windowGap) => {
                    setGlide({ ...glide, windowGap });
                  }}
                  unit="px"
                  value={glide.windowGap}
                />
              </div>
            )}
          </ControlRow>
          <SwitchRow
            isDisabled={!glide.enabled}
            isSelected={glide.cursorFollows}
            onChange={(cursorFollows) => {
              setGlide({ ...glide, cursorFollows });
            }}
            title="Move pointer with window"
          />
          {isMacHost ? (
            <SwitchRow
              description="Feel a tap when the next move is ready."
              isDisabled={!glide.enabled}
              isSelected={glide.haptics}
              onChange={(haptics) => {
                setGlide({ ...glide, haptics });
              }}
              title="Trackpad feedback"
            />
          ) : null}
          <SwitchRow
            description="Two fingers, or hold mouse control while double-clicking."
            isDisabled={!glide.enabled}
            isSelected={glide.doubleTapCenter}
            onChange={(doubleTapCenter) => {
              setGlide({ ...glide, doubleTapCenter });
            }}
            title="Double-tap or double-click to center"
          />
        </ToolGroup>
      ) : null}
      {ruler ? (
        <ToolGroup art={<RulerArt isOn={ruler.enabled} />} title="Ruler">
          <SwitchRow
            description="Sizes and distances in pixels, anywhere on screen."
            isSelected={ruler.enabled}
            onChange={(enabled) => {
              setRuler({ ...ruler, enabled });
            }}
            title="Measure sizes and distances on your screen"
          />
        </ToolGroup>
      ) : null}
      {annotate ? (
        <ToolGroup
          art={
            <AnnotateArt
              color={annotate.defaultColor}
              isKept={annotate.keepAnnotationsBetweenSessions}
              isOn={annotate.enabled}
            />
          }
          title="Annotate"
        >
          <SwitchRow
            description="Annotations are kept in recordings."
            isSelected={annotate.enabled}
            onChange={(enabled) => {
              setAnnotate({ ...annotate, enabled });
            }}
            title="Draw on your screen"
          />
          <SwitchRow
            description="They stay on screen until you clear them."
            isDisabled={!annotate.enabled}
            isSelected={annotate.keepAnnotationsBetweenSessions}
            onChange={(keepAnnotationsBetweenSessions) => {
              setAnnotate({ ...annotate, keepAnnotationsBetweenSessions });
            }}
            title="Keep annotations after exiting"
          />
        </ToolGroup>
      ) : null}
      {ocr ? (
        <ToolGroup art={<OcrArt isOn={ocr.enabled} />} title="OCR">
          <SwitchRow
            description="Copy text from anything on screen, even pictures."
            isSelected={ocr.enabled}
            onChange={(enabled) => {
              setOcr({ ...ocr, enabled });
            }}
            title="Select text or a QR code on your screen"
          />
        </ToolGroup>
      ) : null}
    </Pane>
  );
}
