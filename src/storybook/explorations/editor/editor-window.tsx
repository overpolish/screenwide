// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";
import { FakeScreen, Scene, Waveform } from "../shared/picture";

import { AnnotationRing } from "./annotation-ring";
import { ArrowPanel } from "./arrow-panel";
import { at, CAPTIONS, MOMENTS, TIMELINE_LABEL_WIDTH } from "./editor-data";
import { MomentNote, TracksMenu } from "./editor-menus";
import { PictureTools } from "./picture-tools";
import { PlaybackBar, PreviewZoom, TimelineTools } from "./playback";
import { Timeline } from "./timeline";
import { MagneticTimeline } from "./timeline-magnetic";
import { type EditorKind, TitleBar } from "./title-bar";

// The editor's content: the window draws its own material, so nothing here
// paints the window. A column of tools beside each of the picture and the
// timeline, the annotations in a ring, and the tool panel hung from the
// workspace's corner. Everything that works the timeline stays at the
// timeline, wherever it sits. The picture sits in a recess, so it reads as
// the work rather than as another control.

// The selected arrow's ends, where its handles sit: the selection on the
// picture, so in the orange.
function EndHandles({ ends }: { ends: { x: number; y: number }[] }) {
  return ends.map(({ x, y }) => (
    <span
      className="absolute size-3 -translate-1/2 rounded-full border-2 border-emphasis bg-white"
      key={`${String(x)}-${String(y)}`}
      style={{ insetInlineStart: `${String(x)}%`, top: `${String(y)}%` }}
    />
  ));
}

/** A recording with no picture: its sound, and its transcript beneath. */
function SoundView() {
  return (
    <div className="flex w-full max-w-[640px] flex-col gap-inset">
      <span className="flex h-28 text-fg-secondary">
        <Waveform bars={120} seed={4} />
      </span>
      <div className="flex flex-col gap-inline text-title font-normal">
        {CAPTIONS.slice(0, 3).map(({ label, start }, index) => (
          <p
            className={cn(index === 1 ? "text-fg" : "text-fg-tertiary")}
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

export function EditorWindow({
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
    <div className="flex min-h-0 grow gap-inline">
      <PictureTools isRingOpen={isRingOpen} kind={kind} />
      <main className="relative flex min-w-0 grow items-center justify-center overflow-hidden rounded-panel bg-recess p-10">
        {kind === "audio" ? (
          <SoundView />
        ) : kind === "screenshot" ? (
          <FakeScreen className="aspect-video w-full max-w-[860px] rounded-part shadow-lg" />
        ) : (
          <Scene
            className="w-full max-w-[720px] rounded-part shadow-lg"
            selection={
              hasPanel ? (ends) => <EndHandles ends={ends} /> : undefined
            }
          />
        )}
        {isRingOpen ? <AnnotationRing /> : null}
        {hasPanel ? (
          <div className="absolute end-inset-sm top-inset-sm">
            <ArrowPanel />
          </div>
        ) : null}
        {hasPicture ? (
          <div className="absolute end-inset-sm bottom-inset-sm">
            <PreviewZoom />
          </div>
        ) : null}
      </main>
    </div>
  );

  const timeline = hasTimeline ? (
    <div className="flex shrink-0 gap-inline">
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
                className="pointer-events-auto absolute bottom-0 -translate-x-1/2"
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
    <div className="flex size-full flex-col text-fg">
      <TitleBar kind={kind} />
      <div className="flex min-h-0 grow flex-col gap-inline px-inset-sm pb-inset-sm">
        {picture}
        {timeline}
      </div>
    </div>
  );
}
