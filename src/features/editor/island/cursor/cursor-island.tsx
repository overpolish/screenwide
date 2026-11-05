// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MousePointerClick } from "lucide-react";
import { type ReactNode } from "react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { DEFAULT_CURSOR_EFFECTS } from "../../export/recording-export-settings";
import {
  AnnotationChoiceRow,
  AnnotationOptions,
} from "../annotation/annotation-island-parts";
import { OverlaySizeChoice } from "../overlay-size-choice";
import { ShowGlyph } from "../show-glyph";

import {
  ClickGlyph,
  KeepInsideGlyph,
  MotionBlurGlyph,
  PointerSizeGlyph,
  SmoothGlyph,
} from "./cursor-glyphs";
import { CursorSceneNote } from "./cursor-scene-note";

import type { CursorEffectSettings } from "../../../../bindings/CursorEffectSettings";
import type { EditorIslandContent } from "../editor-island";

// How the recording's cursor is drawn, the whole recording's at once. The
// row shows or hides it, as the Keyboard island does its shortcuts; inside a
// scene, for that scene alone (`RecordingSceneClip.cursor`), as the Screen
// and Camera islands set a scene's own, with a note at the foot saying what
// the scene does. Under the row, where the cursor shows at the playhead:
// its size, a row named as the rows round it are; then whether it moves
// smoothly, blurs as it moves, ripples where it clicks and is kept inside
// the recording, each an animated toggle, two to a line from the section's
// start to its end; then putting it all back as it was. Hidden there, they
// fold away with it, though they still dress the cursor wherever another
// scene shows it. The editor offers the Cursor tool only for a recording
// with cursor movement, so there is always a cursor to dress.

type CursorToggle = Exclude<
  {
    [
      Key in keyof CursorEffectSettings
    ]: CursorEffectSettings[Key] extends boolean ? Key : never;
  }[keyof CursorEffectSettings],
  "bake"
>;

export type CursorIslandProps = {
  /** Whether the cursor shows at the playhead: as the scene there says, or
   * outside every scene the recording's own choice. Its options are offered
   * only where it shows. */
  isCursorShown: boolean;
  /** Whether the playhead is in a scene, whose own showing of the cursor
   * `isCursorShown` and `onShowChange` then are. */
  isInScene: boolean;
  onChange: (settings: CursorEffectSettings) => void;
  onShowChange: (isShown: boolean) => void;
  /** The scene under the playhead's own choice, null where it does as the
   * recording does or there is no scene. */
  sceneCursor: boolean | null;
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
  const hasSection = props.isCursorShown || props.isInScene;
  return {
    actions: (
      <OptionToggle
        glyph={(isOn) => <ShowGlyph isShown={isOn} />}
        isLabelHidden
        isOn={props.isCursorShown}
        label={t("editor-island-cursor-show")}
        onChange={props.onShowChange}
      />
    ),
    key: "cursor",
    section: hasSection ? (
      <IslandParts className="flex flex-col gap-separate">
        {props.isCursorShown ? (
          <AnnotationChoiceRow key="size" title={t("editor-island-size")}>
            <OverlaySizeChoice
              aria-label={t("editor-island-cursor-size")}
              glyph={(side) => <PointerSizeGlyph side={side} />}
              onChange={(size) => {
                onChange({ ...settings, size });
              }}
              value={settings.size}
            />
          </AnnotationChoiceRow>
        ) : null}
        {props.isCursorShown ? (
          <AnnotationOptions key="options">
            {toggle(
              "smoothMovement",
              t("editor-island-cursor-smooth"),
              (isOn) => (
                <SmoothGlyph isSmooth={isOn} />
              ),
            )}
            {toggle(
              "motionBlur",
              t("editor-island-cursor-motion-blur"),
              (isOn) => (
                <MotionBlurGlyph isBlurred={isOn} />
              ),
            )}
            {toggle(
              "clickAnimation",
              t("editor-island-cursor-click-animation"),
              (isOn) => (
                <ClickGlyph isAnimated={isOn} />
              ),
            )}
            {toggle(
              "clipAtVideoEdge",
              t("editor-island-cursor-clip"),
              (isOn) => (
                <KeepInsideGlyph isInside={isOn} />
              ),
            )}
          </AnnotationOptions>
        ) : null}
        {props.isInScene ? (
          <CursorSceneNote
            choice={props.sceneCursor}
            isShown={props.isCursorShown}
            key="scene-note"
          />
        ) : null}
        {props.isCursorShown ? (
          <div className="flex justify-end" key="actions">
            <Button
              onPress={() => {
                onChange(DEFAULT_CURSOR_EFFECTS);
              }}
            >
              {t("editor-island-cursor-reset")}
            </Button>
          </div>
        ) : null}
      </IslandParts>
    ) : undefined,
    title: (
      <IslandRowTitle
        glyph={<MousePointerClick aria-hidden className="size-icon" />}
        title={t("editor-island-cursor")}
      />
    ),
  };
}
