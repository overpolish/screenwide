// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Circle, ImageDown, OctagonAlert } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { IconSwap } from "../../../primitives/icon-swap/icon-swap";
import {
  IslandButton,
  IslandOptions,
  IslandPrimaryButton,
} from "../../../primitives/island/island-controls";
import { IslandRow } from "../../../primitives/island/island-row";

import { InputGlyph, SourceChip } from "./island-glyphs";
import { INPUTS } from "./island-names";
import { canRecord, canScreenshot } from "./island-warning";

import type {
  IslandInputKind,
  IslandPanel,
  RecordingIslandProps,
} from "./island-types";

// The island's row at rest: what is recorded and the inputs to one side,
// the screenshot and Record to the other; on a Mac with a notch, the notch
// between them.

export function RestingRow({
  onToggleInput,
  panel,
  props,
  toggle,
}: {
  onToggleInput: (kind: IslandInputKind) => void;
  panel: IslandPanel;
  props: RecordingIslandProps;
  toggle: (panel: IslandPanel) => void;
}) {
  const { mode } = props;
  const display = props.displays.find(
    ({ id }) => id === props.selectedDisplayId,
  );
  const picture =
    mode === "screen" && display?.thumbnail
      ? {
          id: String(display.id),
          kind: "display" as const,
          src: display.thumbnail,
        }
      : mode === "window" && props.chosenWindow?.icon
        ? {
            id: props.chosenWindow.icon,
            kind: "app" as const,
            src: props.chosenWindow.icon,
          }
        : undefined;
  const record = (
    <IslandPrimaryButton
      aria-label={t("recording-controls-record")}
      isDisabled={!canRecord(props)}
      onPress={props.onRecord}
    >
      <Circle aria-hidden className="size-icon" />
    </IslandPrimaryButton>
  );
  return (
    <IslandRow
      end={
        <>
          <IslandOptions
            isDisabled={!canScreenshot(props)}
            isOpen={panel === "screenshot"}
            label={t("recording-controls-screenshot-options")}
            onOpen={() => {
              toggle("screenshot");
            }}
          >
            <IslandButton
              aria-label={t("recording-controls-screenshot")}
              className={cn(
                props.screenshotState === "pending" && "text-fg-secondary",
              )}
              isDisabled={!canScreenshot(props)}
              onPress={() => props.onScreenshot?.("save")}
            >
              {/* Status is carried by the glyph alone, as in the recording
                  bar, and the glyphs crossfade as it changes. */}
              <IconSwap
                swapKey={
                  props.screenshotState === "done" ||
                  props.screenshotState === "failed"
                    ? props.screenshotState
                    : "ready"
                }
              >
                {props.screenshotState === "done" ? (
                  <span className="flex text-success">
                    <Check aria-hidden className="size-icon" />
                  </span>
                ) : props.screenshotState === "failed" ? (
                  <span className="flex text-error">
                    <OctagonAlert aria-hidden className="size-icon" />
                  </span>
                ) : (
                  <ImageDown
                    aria-hidden
                    className={cn(
                      "size-icon",
                      props.screenshotState === "pending" && "animate-pulse",
                    )}
                  />
                )}
              </IconSwap>
            </IslandButton>
          </IslandOptions>
          {/* Record's options are the replay buffer's, so where it cannot
              run Record stands alone. */}
          {props.replay ? (
            <IslandOptions
              isOpen={panel === "record"}
              label={t("recording-controls-island-record-options")}
              onOpen={() => {
                toggle("record");
              }}
            >
              {record}
            </IslandOptions>
          ) : (
            record
          )}
        </>
      }
      start={
        <>
          <SourceChip
            hasProblem={!canRecord(props)}
            isOpen={panel === "source"}
            mode={mode}
            onPress={() => {
              toggle("source");
            }}
            picture={picture}
          />
          {INPUTS.map((kind) => (
            <InputGlyph
              input={props[kind]}
              isOpen={panel === kind}
              key={kind}
              kind={kind}
              onOpen={() => {
                toggle(kind);
              }}
              onToggle={() => {
                onToggleInput(kind);
              }}
            />
          ))}
        </>
      }
    />
  );
}
