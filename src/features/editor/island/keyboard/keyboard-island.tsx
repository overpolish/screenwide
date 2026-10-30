// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard, Trash2 } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { PositionGrid } from "../../../../primitives/position-grid/position-grid";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { placementOptions } from "../../tool-panels/placement-options";
import {
  AnnotationAnimateToggle,
  AnnotationLine,
} from "../annotation/annotation-island-parts";
import { IslandRowPresence } from "../island-row-presence";
import { OverlaySizeChoice } from "../overlay-size-choice";

import { AppearanceGlyph, KeycapSizeGlyph, ShowGlyph } from "./keyboard-glyphs";

import type { KeyboardEffectSettings } from "../../../../bindings/KeyboardEffectSettings";
import type { EditorIslandContent } from "../editor-island";

// How the shortcuts pressed during the recording are drawn, all of them at
// once: choosing one on the strip opens this island, as there is no styling
// one alone. Its row ends with whether they pop in, as every island's
// Animate does, and whether they show at all; then, with shortcuts chosen
// on the strip, deleting them, which asks twice. Hidden, there is nothing
// else to set, so the section folds away. Under the row: their size and
// whether the keys are dark or light; then where they sit; then, once some
// have been deleted, bringing them all back. The editor offers the Keyboard
// tool only for a recording with shortcuts in it. A recording that fades
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
      <>
        <IslandRowPresence isShown={settings.bake}>
          <AnnotationAnimateToggle
            isAnimated={settings.animation !== "none"}
            onChange={(isAnimated) => {
              onChange({ animation: isAnimated ? "pop" : "none" });
            }}
          />
        </IslandRowPresence>
        <OptionToggle
          glyph={(isOn) => <ShowGlyph isShown={isOn} />}
          isOn={settings.bake}
          label={t("editor-panels-keyboard-show")}
          onChange={(bake) => {
            onChange({ bake });
          }}
        />
        {props.chosen && props.onDeleteChosen ? (
          <ConfirmActionButton
            confirmLabel={t("editor-island-keyboard-confirm-delete")}
            icon={<Trash2 aria-hidden />}
            label={t("editor-island-keyboard-delete", { count: props.chosen })}
            onConfirm={props.onDeleteChosen}
            variant="ghost"
          />
        ) : null}
      </>
    ),
    key: "keyboard",
    section: settings.bake ? (
      <IslandFlow className="flex flex-col gap-separate">
        <AnnotationLine>
          <OverlaySizeChoice
            aria-label={t("editor-panels-size")}
            glyph={(side) => <KeycapSizeGlyph side={side} />}
            onChange={(size) => {
              onChange({ size });
            }}
            value={settings.size}
          />
          <OptionToggle
            glyph={(isOn) => <AppearanceGlyph isLight={isOn} />}
            isOn={settings.appearance === "light"}
            label={t("editor-panels-keyboard-light")}
            onChange={(isLight) => {
              onChange({ appearance: isLight ? "light" : "dark" });
            }}
          />
        </AnnotationLine>
        <AnnotationLine>
          <PositionGrid
            aria-label={t("editor-panels-position")}
            items={placementOptions()}
            onChange={(position) => {
              onChange({ position });
            }}
            value={settings.position}
          />
        </AnnotationLine>
        {props.canRestoreShortcuts ? (
          <div className="flex justify-end">
            <Button onPress={props.onRestoreShortcuts}>
              {t("editor-panels-keyboard-restore")}
            </Button>
          </div>
        ) : null}
      </IslandFlow>
    ) : undefined,
    title: (
      <IslandRowTitle
        glyph={<Keyboard aria-hidden className="size-icon" />}
        title={t("editor-panels-keyboard")}
      />
    ),
  };
}
