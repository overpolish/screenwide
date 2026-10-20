// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import "./combined.css";

import { cn } from "../../../../lib/styling";
import { FakeScreen, Scene, Waveform } from "../shared/mock-parts";

import { type EditorKind, PictureTools, TitleBar } from "./combined-chrome";
import { at, CAPTIONS, MOMENTS, TIMELINE_LABEL_WIDTH } from "./combined-data";
import { ArrowPanel, MomentNote, TracksMenu } from "./combined-panels";
import { PlaybackBar, PreviewZoom, TimelineTools } from "./combined-playback";
import { AnnotationRing } from "./combined-ring";
import { Timeline } from "./combined-timeline";
import { MagneticTimeline } from "./timeline-magnetic";

// The direction the explorations converged on: Polaris's controls with a
// quieter bevel, a column of tools beside each of the picture and the
// timeline, the annotations in a ring, and today's tool panel hung from the
// workspace's corner. Everything that works the timeline stays at the
// timeline, wherever it sits.

function EndHandles({ ends }: { ends: { x: number; y: number }[] }) {
  return ends.map(({ x, y }) => (
    <span
      className="absolute size-3 -translate-1/2 rounded-full border-2 border-(--emphasis) bg-white"
      key={`${String(x)}-${String(y)}`}
      style={{ insetInlineStart: `${String(x)}%`, top: `${String(y)}%` }}
    />
  ));
}

/** A recording with no picture: its sound, and its transcript beneath. */
function SoundView() {
  return (
    <div className="flex w-full max-w-[640px] flex-col gap-6">
      <span className="flex h-28 text-(--level)">
        <Waveform bars={120} seed={4} />
      </span>
      <div className="flex flex-col gap-2 text-[15px] leading-6">
        {CAPTIONS.slice(0, 3).map(({ label, start }, index) => (
          <p
            className={cn(index === 1 ? "text-(--fg)" : "text-(--fg-3)")}
            key={start}
          >
            {label}.
          </p>
        ))}
      </div>
    </div>
  );
}

const NOTE_AT = MOMENTS.find(({ note }) => note)?.at ?? 0;

export function CombinedEditor({
  hasPanel = false,
  isMagnetic = false,
  isNoteOpen = false,
  isRingOpen = false,
  isTracksOpen = false,
  kind = "recording",
}: {
  hasPanel?: boolean;
  /** One storyline with connected clips, Final Cut's way, instead of rows. */
  isMagnetic?: boolean;
  isNoteOpen?: boolean;
  isRingOpen?: boolean;
  isTracksOpen?: boolean;
  kind?: EditorKind;
}) {
  const hasPicture = kind === "recording";
  const hasTimeline = kind !== "screenshot";

  const picture = (
    <div className="flex min-h-0 grow gap-1.5">
      <PictureTools isRingOpen={isRingOpen} kind={kind} />
      <main className="relative flex min-w-0 grow items-center justify-center overflow-hidden rounded-xl bg-(--recess) p-10">
        {kind === "audio" ? (
          <SoundView />
        ) : kind === "screenshot" ? (
          <FakeScreen className="aspect-video w-full max-w-[860px] rounded-md shadow-lg" />
        ) : (
          <Scene
            className="w-full max-w-[720px] rounded-md shadow-lg"
            selection={
              hasPanel ? (ends) => <EndHandles ends={ends} /> : undefined
            }
          />
        )}
        {isRingOpen ? <AnnotationRing /> : null}
        {hasPanel ? (
          <div className="absolute end-2 top-2">
            <ArrowPanel />
          </div>
        ) : null}
        {hasPicture ? (
          <div className="absolute end-2 bottom-2">
            <PreviewZoom />
          </div>
        ) : null}
      </main>
    </div>
  );

  const timeline = hasTimeline ? (
    <div className="flex shrink-0 gap-1.5">
      <TimelineTools menu={isTracksOpen ? <TracksMenu /> : undefined} />
      <section className="relative flex min-w-0 grow flex-col">
        <PlaybackBar hasPicture={hasPicture} />
        {isMagnetic ? (
          <MagneticTimeline hasPicture={hasPicture} />
        ) : (
          <Timeline hasPicture={hasPicture} />
        )}
        {isNoteOpen ? (
          // The note opens from its pin toward the picture.
          <div
            className="pointer-events-none absolute inset-x-0 bottom-full mb-1 pe-3.5"
            dir="ltr"
            style={{
              paddingInlineStart: isMagnetic ? 0 : TIMELINE_LABEL_WIDTH,
            }}
          >
            <div className="relative">
              <div
                className="absolute bottom-0 -translate-x-1/2"
                style={{ insetInlineStart: at(NOTE_AT) }}
              >
                <MomentNote />
              </div>
            </div>
          </div>
        ) : null}
      </section>
    </div>
  ) : null;

  return (
    <div className="look-combined flex size-full flex-col font-sans text-(--fg)">
      <TitleBar kind={kind} />
      <div className="flex min-h-0 grow flex-col gap-1.5 px-2 pb-2">
        {picture}
        {timeline}
      </div>
    </div>
  );
}
