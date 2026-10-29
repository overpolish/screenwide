// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { PositionGrid } from "../../../../primitives/position-grid/position-grid";
import { CAPTION_WORDS } from "../../recording/captions/recording-captions";
import { placementOptions } from "../../tool-panels/placement-options";
import { AnnotationLine } from "../annotation/annotation-island-parts";
import { TextColors } from "../annotation/text/text-colors";
import { TextLookPickers } from "../annotation/text/text-look-pickers";

import {
  AllCapsToggle,
  CaptionLinesChoice,
  CaptionSizeChoice,
  CaptionWordToggle,
} from "./caption-choices";

import type { CaptionsIslandProps } from "./captions-island";

/** The captions' section once there is a transcript and they show: the
 * text's pickers, with the captions' layout where a text's placement goes
 * and what they say among the lines under the faces. */
export function CaptionsSection(props: CaptionsIslandProps) {
  const { look, onLayoutChange } = props;
  return (
    <TextLookPickers
      families={props.families}
      look={look}
      onLookChange={props.onLookChange}
      placement={
        // Parts that come and go with the template (the key word's size,
        // the line count of captions set in lines) do so in place, the rest
        // gliding to make room.
        <IslandFlow className="flex flex-col gap-separate">
          <AnnotationLine key="size">
            <CaptionSizeChoice
              key="size"
              onChange={(size) => {
                onLayoutChange({ size });
              }}
              value={look.size}
            />
            {look.accent === "word" ? (
              <CaptionSizeChoice
                isAccent
                key="accent-size"
                onChange={(accentSize) => {
                  onLayoutChange({ accentSize });
                }}
                value={look.accentSize}
              />
            ) : null}
          </AnnotationLine>
          <AnnotationLine key="place">
            <PositionGrid
              aria-label={t("editor-island-captions-position")}
              items={placementOptions()}
              onChange={(position) => {
                onLayoutChange({ position });
              }}
              value={look.position}
            />
            {/* Beside where they sit, how much a caption holds: its lines
                over its words, the two together as tall as the grid. */}
            <IslandFlow
              className="flex flex-col items-end gap-inline"
              key="fill"
            >
              {/* A stack or a staircase breaks its own lines, so the count
                  only holds for captions set in lines. */}
              {look.arrangement === "lines" ? (
                <CaptionLinesChoice
                  key="lines"
                  onChange={(lines) => {
                    onLayoutChange({ lines });
                  }}
                  value={look.lines}
                />
              ) : null}
              <NumberField
                aria-label={t("editor-island-captions-max-words")}
                key="max-words"
                leftSection={t("editor-island-captions-max")}
                maxValue={CAPTION_WORDS.maximum}
                minValue={CAPTION_WORDS.minimum}
                onChange={(maxWords) => {
                  onLayoutChange({ maxWords });
                }}
                rightSection={t("editor-island-captions-words-unit", {
                  count: look.maxWords,
                })}
                step={1}
                value={look.maxWords}
              />
            </IslandFlow>
          </AnnotationLine>
        </IslandFlow>
      }
      still={false}
      templates={props.templates}
    >
      <AnnotationLine key="words">
        {(["hold", "punctuation", "fillers", "profanity"] as const).map(
          (rule) => (
            <CaptionWordToggle
              key={rule}
              onChange={props.onSettingsChange}
              rule={rule}
              settings={props.settings}
            />
          ),
        )}
        <AllCapsToggle
          allCaps={look.allCaps}
          key="all-caps"
          onChange={(allCaps) => {
            onLayoutChange({ allCaps });
          }}
        />
      </AnnotationLine>
      <TextColors
        key="colors"
        look={look}
        onLookChange={props.onLookChange}
        onRemoveColor={props.onRemoveColor}
        onSaveColor={props.onSaveColor}
        savedColors={props.savedColors}
      />
      {/* One-shot actions, at the end of their own row and as wide as their
          labels; bringing the captions back only while some are out. They
          take the width the lines above set rather than widening the
          island, and wrap within it. */}
      <div
        className="flex w-0 min-w-full flex-wrap justify-end gap-inline"
        key="actions"
      >
        {props.hiddenCount > 0 ? (
          <Button onPress={props.onRestoreHidden}>
            {t("editor-island-captions-restore", {
              count: props.hiddenCount,
            })}
          </Button>
        ) : null}
        <Button isDisabled={!props.canReset} onPress={props.onReset}>
          {t("editor-island-captions-reset")}
        </Button>
        <Button isDisabled={!props.canSetDefault} onPress={props.onSetDefault}>
          {t("editor-island-captions-set-default")}
        </Button>
      </div>
    </TextLookPickers>
  );
}
