// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, useState } from "react";

import { Button } from "../../../primitives/button/button";
import { ProgressBar } from "../../../primitives/progress-bar/progress-bar";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { Switch } from "../../../primitives/switch/switch";
import { SettingRow } from "../shared/setting-row";

import {
  BALANCED,
  CAMERA_STEPS,
  type QualityStep,
  SCREEN_STEPS,
} from "./export-data";
import {
  CameraFrame,
  Choose,
  ComparePreview,
  FileLine,
  SoundPreview,
} from "./export-parts";
import { Quality } from "./export-quality";

// The export window's content, built around the file it makes rather than a
// form of its settings: the frame as it will come out, the name and place to
// save it, one quality choice that shows its result, and the rest as
// switches, each a setting the app already has. The body scrolls under a
// fixed foot, so Export is always in reach.

export type ExportKind = "recording" | "screenshot" | "sound";
export type ExportSaving = "confirm-cancel" | "progress";

/** A file of a recording saved as two: its frame, what it is, its quality. */
function FileCard({
  label,
  picture,
  steps,
}: {
  label: string;
  picture: ReactNode;
  steps: QualityStep[];
}) {
  const step = steps[BALANCED];
  return (
    <div className="flex min-w-0 flex-col gap-inline">
      {picture}
      <span className="flex items-baseline justify-between gap-inline">
        <span className="text-label">{label}</span>
        <span className="flex gap-inline text-caption text-fg-tertiary tabular-nums">
          <span>{step.dimensions}</span>
          <span>{step.size}</span>
        </span>
      </span>
      <Choose
        className="w-full"
        items={steps.map(({ label: id }) => ({ id, label: id }))}
        label={`${label} quality`}
        selected={step.label}
      />
    </div>
  );
}

function Option({
  description,
  initial = false,
  isDisabled,
  title,
}: {
  title: string;
  description?: string;
  initial?: boolean;
  isDisabled?: boolean;
}) {
  const [isOn, setIsOn] = useState(initial);
  return (
    <SettingRow description={description} title={title}>
      {(labelledBy) => (
        <Switch
          aria-labelledby={labelledBy}
          isDisabled={isDisabled}
          isSelected={isOn && !isDisabled}
          onChange={setIsOn}
        />
      )}
    </SettingRow>
  );
}

// Saving shows its progress where the settings were. Cancelling throws the
// partial file away, so it asks first, in words, with keeping on as the
// primary choice.
function Saving({ isConfirming }: { isConfirming: boolean }) {
  return (
    <section className="flex flex-col gap-inline">
      <span className="flex items-baseline justify-between">
        <span className="text-label">Saving camera</span>
        <span className="text-caption text-fg-tertiary">
          Less than a minute remaining
        </span>
      </span>
      <ProgressBar aria-label="Saving" value={42} />
      <span className="mt-inline flex items-center justify-end gap-inline">
        {isConfirming ? (
          <>
            <span className="me-auto text-body text-fg-secondary">
              Stop and delete what is saved so far?
            </span>
            <Button>Stop</Button>
            <Button color="primary">Keep saving</Button>
          </>
        ) : (
          <Button>Cancel</Button>
        )}
      </span>
    </section>
  );
}

export function ExportWindow({
  hasSeparateCamera = false,
  hasTranscript = true,
  isCustomised = false,
  kind = "recording",
  saving,
}: {
  /** The camera saved as a file of its own, with its own quality. */
  hasSeparateCamera?: boolean;
  hasTranscript?: boolean;
  /** Size and compression set apart, instead of a quality step. */
  isCustomised?: boolean;
  kind?: ExportKind;
  saving?: ExportSaving;
}) {
  const isRecording = kind !== "screenshot";
  const hasPicture = kind !== "sound";
  const extension =
    kind === "screenshot" ? "png" : kind === "sound" ? "m4a" : "mp4";
  const size =
    kind === "screenshot"
      ? "2.1 MB"
      : kind === "sound"
        ? "6 MB"
        : hasSeparateCamera
          ? "96 MB"
          : undefined;

  const preview =
    kind === "sound" ? (
      <SoundPreview />
    ) : hasSeparateCamera ? (
      <div className="grid grid-cols-2 gap-separate">
        <FileCard
          label="Screen"
          picture={<ComparePreview isComparing={false} />}
          steps={SCREEN_STEPS}
        />
        <FileCard
          label="Camera"
          picture={<CameraFrame />}
          steps={CAMERA_STEPS}
        />
      </div>
    ) : (
      <ComparePreview isComparing={kind === "recording" && !saving} />
    );

  return (
    <div className="flex size-full flex-col text-fg">
      <header className="flex h-12 shrink-0 items-center ps-traffic-lights pe-inset text-heading">
        Export
      </header>
      <ScrollArea
        className="flex flex-col gap-separate px-inset pb-inset"
        rootClassName="min-h-0 grow"
      >
        {preview}
        <FileLine extension={extension} size={size} />
        {saving ? (
          <Saving isConfirming={saving === "confirm-cancel"} />
        ) : (
          <>
            {hasPicture && isRecording && !hasSeparateCamera ? (
              <Quality initialCustomised={isCustomised} steps={SCREEN_STEPS} />
            ) : null}
            <div className="flex flex-col gap-inline">
              {isRecording ? (
                <>
                  <Option
                    description={
                      hasTranscript
                        ? "An SRT file beside the video"
                        : "Transcribe the recording to add subtitles."
                    }
                    initial
                    isDisabled={!hasTranscript}
                    title="Subtitles"
                  />
                  <Option
                    description="One sound track instead of two"
                    title="Mix audio tracks"
                  />
                </>
              ) : null}
              <Option initial title="Open folder after saving" />
              <Option title="Delete project after saving" />
            </div>
          </>
        )}
      </ScrollArea>
      {saving ? null : (
        <footer className="flex shrink-0 justify-end gap-inline px-inset pb-inset">
          <Button>Cancel</Button>
          <Button color="primary">Export</Button>
        </footer>
      )}
    </div>
  );
}
