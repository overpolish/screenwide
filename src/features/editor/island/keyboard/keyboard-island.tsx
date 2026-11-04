// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard, Trash2 } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { PositionGrid } from "../../../../primitives/position-grid/position-grid";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { placementOptions } from "../../tools/placement-options";
import {
  AnnotationAnimateToggle,
  AnnotationChoiceRow,
  AnnotationLine,
} from "../annotation/annotation-island-parts";
import { OverlaySizeChoice } from "../overlay-size-choice";

import { AppearanceGlyph, KeycapSizeGlyph, ShowGlyph } from "./keyboard-glyphs";

import type { KeyboardEffectSettings } from "../../../../bindings/KeyboardEffectSettings";
import type { EditorIslandContent } from "../editor-island";

// How the shortcuts pressed during the recording are drawn, all of them at
// once: choosing one on the strip opens this island, as there is no styling
// one alone. Its row ends with whether they pop in, as every island's
// Animate does, and whether they show at all; then, with shortcuts chosen
// on the strip, deleting them, which asks twice. Hidden, there is nothing
// else to set, so the section folds away. Under the row: their size, a row
// named as the rows round it are; then where they sit, with whether the
// keys are dark or light at that line's end; then, once some have been
// deleted, bringing them all back. The editor offers the Keyboard tool
// only for a recording with shortcuts in it. A recording that fades
// its shortcuts in shows as animated and keeps its fade until Animate is
// turned off and on.

export type KeyboardIslandProps = {
  /** Whether any shortcuts have been deleted, to bring back. */
  canRestoreShortcuts: boolean;
  onChange: (patch: Partial<KeyboardEffectSettings>) => void;
  onRestoreShortcuts: () => void;
  settings: KeyboardEffectSettings;
  /** How many shortcuts are chosen on the strip, to delete. */
  chosen?: number;
  /** Told to delete the shortcuts chosen on the strip. */
  onDeleteChosen?: () => void;
};

export function keyboardIsland(
  props: KeyboardIslandProps,
): EditorIslandContent {
  const { onChange, settings } = props;
  return {
    actions: (
      <IslandParts className="flex items-center gap-separate" direction="row">
        {settings.bake ? (
          <AnnotationAnimateToggle
            isAnimated={settings.animation !== "none"}
            key="animate"
            onChange={(isAnimated) => {
              onChange({ animation: isAnimated ? "pop" : "none" });
            }}
          />
        ) : null}
        <OptionToggle
          glyph={(isOn) => <ShowGlyph isShown={isOn} />}
          isLabelHidden
          isOn={settings.bake}
          key="show"
          label={t("editor-island-keyboard-show")}
          onChange={(bake) => {
            onChange({ bake });
          }}
        />
        {props.chosen && props.onDeleteChosen ? (
          <ConfirmActionButton
            confirmLabel={t("editor-island-keyboard-confirm-delete")}
            icon={<Trash2 aria-hidden />}
            key="delete"
            label={t("editor-island-keyboard-delete", { count: props.chosen })}
            onConfirm={props.onDeleteChosen}
            variant="ghost"
          />
        ) : null}
      </IslandParts>
    ),
    key: "keyboard",
    section: settings.bake ? (
      <IslandParts className="flex flex-col gap-separate">
        <AnnotationChoiceRow key="size" title={t("editor-island-size")}>
          <OverlaySizeChoice
            aria-label={t("editor-island-size")}
            glyph={(side) => <KeycapSizeGlyph side={side} />}
            onChange={(size) => {
              onChange({ size });
            }}
            value={settings.size}
          />
        </AnnotationChoiceRow>
        <AnnotationLine key="place">
          <PositionGrid
            aria-label={t("editor-island-position")}
            items={placementOptions()}
            onChange={(position) => {
              onChange({ position });
            }}
            value={settings.position}
          />
          <OptionToggle
            glyph={(isOn) => <AppearanceGlyph isLight={isOn} />}
            isOn={settings.appearance === "light"}
            label={t("editor-island-keyboard-light")}
            onChange={(isLight) => {
              onChange({ appearance: isLight ? "light" : "dark" });
            }}
          />
        </AnnotationLine>
        {props.canRestoreShortcuts ? (
          <div className="flex justify-end" key="restore">
            <Button onPress={props.onRestoreShortcuts}>
              {t("editor-island-keyboard-restore")}
            </Button>
          </div>
        ) : null}
      </IslandParts>
    ) : undefined,
    title: (
      <IslandRowTitle
        glyph={<Keyboard aria-hidden className="size-icon" />}
        title={t("editor-island-keyboard")}
      />
    ),
  };
}
