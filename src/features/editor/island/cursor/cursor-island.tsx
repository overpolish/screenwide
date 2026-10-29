// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MousePointer2 } from "lucide-react";
import { type ReactNode } from "react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { DEFAULT_CURSOR_EFFECTS } from "../../export/recording-export-settings";
import { AnnotationLine } from "../annotation/annotation-island-parts";
import { OverlaySizeChoice } from "../overlay-size-choice";

import {
  ClickGlyph,
  KeepInsideGlyph,
  MotionBlurGlyph,
  PointerSizeGlyph,
  SmoothGlyph,
} from "./cursor-glyphs";

import type { CursorEffectSettings } from "../../../../bindings/CursorEffectSettings";
import type { EditorIslandContent } from "../editor-island";

// How the recording's cursor is drawn, the whole recording's at once. Under
// the row: its size; then whether it moves smoothly, blurs as it moves,
// ripples where it clicks and is kept inside the recording, each an
// animated toggle; then putting it all back as it was. The editor offers
// the Cursor tool only for a recording with cursor movement, so there is
// always a cursor to dress.

type CursorToggle = Exclude<
  {
    [
      Key in keyof CursorEffectSettings
    ]: CursorEffectSettings[Key] extends boolean ? Key : never;
  }[keyof CursorEffectSettings],
  "bake"
>;

export type CursorIslandProps = {
  onChange: (settings: CursorEffectSettings) => void;
  settings: CursorEffectSettings;
};

export function cursorIsland(props: CursorIslandProps): EditorIslandContent {
  const { onChange, settings } = props;
  const toggle = (
    key: CursorToggle,
    label: string,
    glyph: (isOn: boolean) => ReactNode,
  ) => (
    <OptionToggle
      glyph={glyph}
      isOn={settings[key]}
      label={label}
      onChange={(isOn) => {
        onChange({ ...settings, [key]: isOn });
      }}
    />
  );
  return {
    key: "cursor",
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationLine>
          <OverlaySizeChoice
            aria-label={t("editor-panels-cursor-size")}
            glyph={(side) => <PointerSizeGlyph side={side} />}
            onChange={(size) => {
              onChange({ ...settings, size });
            }}
            value={settings.size}
          />
        </AnnotationLine>
        <AnnotationLine>
          {toggle(
            "smoothMovement",
            t("editor-panels-cursor-smooth"),
            (isOn) => (
              <SmoothGlyph isSmooth={isOn} />
            ),
          )}
          {toggle(
            "motionBlur",
            t("editor-panels-cursor-motion-blur"),
            (isOn) => (
              <MotionBlurGlyph isBlurred={isOn} />
            ),
          )}
          {toggle(
            "clickAnimation",
            t("editor-panels-cursor-click-animation"),
            (isOn) => (
              <ClickGlyph isAnimated={isOn} />
            ),
          )}
          {toggle("clipAtVideoEdge", t("editor-panels-cursor-clip"), (isOn) => (
            <KeepInsideGlyph isInside={isOn} />
          ))}
        </AnnotationLine>
        <div className="flex justify-end">
          <Button
            onPress={() => {
              onChange(DEFAULT_CURSOR_EFFECTS);
            }}
          >
            {t("editor-island-cursor-reset")}
          </Button>
        </div>
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<MousePointer2 aria-hidden className="size-icon" />}
        title={t("editor-panels-cursor")}
      />
    ),
  };
}
