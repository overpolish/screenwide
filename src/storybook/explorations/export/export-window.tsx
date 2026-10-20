// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import "../direction/combined/combined.css";

import { Captions, ChevronDown, FolderOpen, Merge, Trash2 } from "lucide-react";
import { type ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { Btn, Row } from "../direction/combined/combined-controls";

import {
  BALANCED,
  CAMERA_STEPS,
  type QualityStep,
  SCREEN_STEPS,
} from "./export-data";
import {
  CameraFrame,
  Chip,
  ChipLine,
  ComparePreview,
  Destination,
  FileTitle,
  QualitySteps,
  SoundPreview,
} from "./export-parts";

// The export window in the unified look, built around the file it makes
// rather than a form of its settings: the frame as it will come out, the
// name and place to save it, one quality choice that shows its result, and
// the rest as chips. Every choice is one the app already has.

export type ExportKind = "recording" | "screenshot" | "sound";
export type ExportSaving = "confirm-cancel" | "progress";

function Select({ children }: { children: ReactNode }) {
  return (
    <Btn className="min-w-36 justify-between">
      {children}
      <ChevronDown className="opacity-60" />
    </Btn>
  );
}

/** Today's separate size and compression, for when no step fits. */
function CustomQuality() {
  return (
    <div className="flex flex-col gap-1 rounded-xl bg-(--fill) px-3 py-1.5">
      <Row isChanged label="Size">
        <Select>
          <span className="tabular-nums">2880 × 1620</span>
        </Select>
      </Row>
      <Row label="Compression">
        <Select>Balanced</Select>
      </Row>
    </div>
  );
}

function Quality({ isCustomised }: { isCustomised: boolean }) {
  return (
    <section className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between">
        <span className="text-[12px] font-medium text-(--fg-2)">Quality</span>
        <Btn variant="ghost">{isCustomised ? "Use a preset" : "Customise"}</Btn>
      </div>
      {isCustomised ? (
        <CustomQuality />
      ) : (
        <QualitySteps selected={BALANCED} steps={SCREEN_STEPS} />
      )}
    </section>
  );
}

/** A file of a recording saved as two: its frame, what it is, its size. */
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
    <div className="flex min-w-0 flex-col gap-2">
      {picture}
      <span className="flex items-baseline justify-between gap-2 text-[12px]">
        <span className="font-medium">{label}</span>
        <span className="text-(--fg-3) tabular-nums">
          {step.dimensions} · {step.size}
        </span>
      </span>
      <Select>{step.label}</Select>
    </div>
  );
}

function Saving({ isConfirming }: { isConfirming: boolean }) {
  return (
    <section className="flex items-center gap-3 rounded-xl bg-(--fill) p-3">
      <div className="flex min-w-0 grow flex-col gap-1.5">
        <span className="text-[12px] font-medium">Saving camera</span>
        <span className="h-1 overflow-hidden rounded-full bg-(--fill-2)">
          <span className="block h-full w-[42%] rounded-full bg-(--emphasis)" />
        </span>
        <span className="text-[11px] text-(--fg-3)">
          Less than a minute remaining
        </span>
      </div>
      {/* The raised material outranks a utility background, so the armed
          state starts from the plain button. */}
      <Btn
        className={cn(isConfirming && "bg-(--record) text-white")}
        variant={isConfirming ? "ghost" : "default"}
      >
        {isConfirming ? "Confirm cancel" : "Cancel"}
      </Btn>
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
          ? "Two files, 96 MB"
          : SCREEN_STEPS[BALANCED].size;

  const preview =
    kind === "sound" ? (
      <SoundPreview />
    ) : hasSeparateCamera ? (
      <div className="grid grid-cols-2 gap-3">
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
    <div className="look-combined flex size-full flex-col font-sans text-(--fg)">
      <div className="flex h-9 shrink-0 items-center ps-traffic-lights pe-3 text-[12px] font-medium text-(--fg-2)">
        Export
      </div>
      <div className="flex min-h-0 grow flex-col gap-3 px-3 pb-3">
        {preview}
        <div className="flex flex-col gap-0.5">
          <FileTitle extension={extension} />
          <Destination size={size} />
        </div>
        {saving ? (
          <Saving isConfirming={saving === "confirm-cancel"} />
        ) : (
          <>
            {hasPicture && isRecording && !hasSeparateCamera ? (
              <Quality isCustomised={isCustomised} />
            ) : null}
            <div className="flex flex-col gap-1.5">
              {isRecording ? (
                <ChipLine label="Include">
                  <Chip
                    hasMenu={hasTranscript}
                    icon={Captions}
                    isDisabled={!hasTranscript}
                    isOn={hasTranscript}
                  >
                    {hasTranscript ? "Subtitles, SRT" : "Subtitles"}
                  </Chip>
                  <Chip icon={Merge}>Mix audio tracks</Chip>
                </ChipLine>
              ) : null}
              {hasTranscript ? null : (
                <span className="ps-[82px] text-[11px] text-(--fg-3)">
                  Transcribe the recording to add subtitles.
                </span>
              )}
              <ChipLine label="After saving">
                <Chip icon={FolderOpen} isOn>
                  Open folder
                </Chip>
                <Chip icon={Trash2}>Delete project</Chip>
              </ChipLine>
            </div>
            <div className="mt-auto flex justify-end gap-2">
              <Btn>Cancel</Btn>
              <Btn variant="primary">Export</Btn>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
