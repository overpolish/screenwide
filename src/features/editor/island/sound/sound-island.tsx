// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Volume2 } from "lucide-react";
import { type ReactNode } from "react";

import { t } from "../../../../i18n/i18n";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { AnnotationLine } from "../annotation/annotation-island-parts";
import { IslandModelLine } from "../island-model-line";
import { IslandWorkLine } from "../island-work-line";

import {
  AutoVolumeGlyph,
  ReduceNoiseGlyph,
  StudioDuckingGlyph,
  StudioSoundGlyph,
  VocalCleanupGlyph,
} from "./sound-glyphs";
import { SoundVolumeRow } from "./sound-volume-row";

import type {
  ToolPanelMicrophone,
  ToolPanelModel,
} from "../../tool-panels/tool-panel-selection";
import type { EditorIslandContent } from "../editor-island";

// The recording's sound, every track at once, as the Cursor and Keyboard
// islands hold their overlays. Each track is a block: how loud it plays, a
// typed figure named by the track, with its reset while it is off the
// recorded level; and for the microphone its tools under it on one line,
// each an animated toggle drawing its off and on looks. Studio sound rebuilds
// the voice in place of Reduce noise and Vocal cleanup, so while it is on
// those two step aside; Studio ducking changes how the system audio makes
// way, which it does only while Auto volume is on, so it shows only then. A
// tool whose model is not on this computer yet offers the download in its
// place, and a tool at work shows how far it has got, each on a line that
// comes and goes as the others glide (`IslandFlow`). Remove silences is not
// here: it sits with the pauses on the strip and with Transcript.

/** A microphone tool that can be turned on. */
export type SoundTool =
  | "autoVolume"
  | "reduceNoise"
  | "studioDucking"
  | "studioSound"
  | "vocalCleanup";

export type SoundTrack = {
  /** How much it is lifted or lowered, 0 being the recorded level. */
  decibels: number;
  id: string;
  /** The track's own name, such as Microphone or System audio. */
  label: string;
  /** The speech tools, for a microphone only. */
  microphone?: Omit<ToolPanelMicrophone, "silences">;
};

export type SoundIslandProps = {
  onDownloadStudioDucking: () => void;
  onDownloadStudioSound: () => void;
  onToolChange: (tool: SoundTool, isOn: boolean) => void;
  onVolumeChange: (trackId: string, decibels: number) => void;
  tracks: SoundTrack[];
};

const isDownloaded = (model: ToolPanelModel | null) =>
  model?.status === "downloaded";

const TOOL_NAMES: Record<SoundTool, () => string> = {
  autoVolume: () => t("editor-panels-auto-volume"),
  reduceNoise: () => t("editor-panels-reduce-noise"),
  studioDucking: () => t("editor-panels-studio-ducking"),
  studioSound: () => t("editor-panels-studio-sound"),
  vocalCleanup: () => t("editor-panels-vocal-cleanup"),
};

export function soundIsland(props: SoundIslandProps): EditorIslandContent {
  return {
    key: "sound",
    section: (
      <IslandFlow className="flex flex-col gap-separate">
        {props.tracks.flatMap((track) => [
          <SoundVolumeRow
            decibels={track.decibels}
            key={`${track.id}-volume`}
            label={track.label}
            onChange={(decibels) => {
              props.onVolumeChange(track.id, decibels);
            }}
          />,
          ...(track.microphone
            ? microphoneParts(track.id, track.microphone, props)
            : []),
        ])}
      </IslandFlow>
    ),
    title: (
      <IslandRowTitle
        glyph={<Volume2 aria-hidden className="size-icon" />}
        title={t("editor-island-sound")}
      />
    ),
  };
}

/** The microphone's tools under its volume: one line of toggles, then a
 * line for each model to download and each tool at work. */
function microphoneParts(
  trackId: string,
  microphone: Omit<ToolPanelMicrophone, "silences">,
  props: SoundIslandProps,
): ReactNode[] {
  const hasStudio = isDownloaded(microphone.studioModel);
  const isStudio = hasStudio && microphone.studio !== "off";
  // Ducking only where the recording has system audio to make way, while
  // Auto volume is on, and once the app has said where its model is.
  const { ducking } = microphone;
  const shownDucking =
    ducking?.model && microphone.autoVolume !== "off"
      ? { ...ducking, model: ducking.model }
      : null;
  const toggle = (
    tool: SoundTool,
    state: "cleaning" | "off" | "on",
    glyph: (isOn: boolean) => ReactNode,
  ) => (
    <OptionToggle
      glyph={glyph}
      isOn={state !== "off"}
      key={tool}
      label={TOOL_NAMES[tool]()}
      onChange={(isOn) => {
        props.onToolChange(tool, isOn);
      }}
    />
  );
  const work = (key: string, label: string, value: number) => (
    <IslandWorkLine key={`${trackId}-${key}`} label={label} value={value} />
  );
  return [
    <AnnotationLine key={`${trackId}-tools`}>
      {hasStudio
        ? toggle("studioSound", microphone.studio, (isOn) => (
            <StudioSoundGlyph isOn={isOn} />
          ))
        : null}
      {isStudio
        ? null
        : toggle("reduceNoise", microphone.noise, (isOn) => (
            <ReduceNoiseGlyph isOn={isOn} />
          ))}
      {isStudio
        ? null
        : toggle("vocalCleanup", microphone.voice, (isOn) => (
            <VocalCleanupGlyph isOn={isOn} />
          ))}
      {toggle("autoVolume", microphone.autoVolume, (isOn) => (
        <AutoVolumeGlyph isOn={isOn} />
      ))}
      {shownDucking && isDownloaded(shownDucking.model)
        ? toggle("studioDucking", shownDucking.state, (isOn) => (
            <StudioDuckingGlyph isOn={isOn} />
          ))
        : null}
    </AnnotationLine>,
    microphone.studioModel && !hasStudio ? (
      <IslandModelLine
        key={`${trackId}-studio-model`}
        model={microphone.studioModel}
        onDownload={props.onDownloadStudioSound}
        title={t("editor-panels-studio-sound")}
      />
    ) : null,
    shownDucking && !isDownloaded(shownDucking.model) ? (
      <IslandModelLine
        key={`${trackId}-ducking-model`}
        model={shownDucking.model}
        onDownload={props.onDownloadStudioDucking}
        title={t("editor-panels-studio-ducking")}
      />
    ) : null,
    microphone.studio === "cleaning"
      ? work(
          "studio-work",
          t("editor-panels-studio-sound-working"),
          microphone.studioProgress,
        )
      : null,
    !isStudio && microphone.noise === "cleaning"
      ? work(
          "noise-work",
          t("editor-panels-noise-cleaning"),
          microphone.noiseProgress,
        )
      : null,
    !isStudio && microphone.voice === "cleaning"
      ? work(
          "voice-work",
          t("editor-panels-voice-cleaning"),
          microphone.voiceProgress,
        )
      : null,
    microphone.autoVolume === "cleaning"
      ? work(
          "volume-work",
          t("editor-panels-auto-volume-measuring"),
          microphone.autoVolumeProgress,
        )
      : null,
    shownDucking?.state === "cleaning"
      ? work(
          "ducking-work",
          t("editor-panels-studio-ducking-working"),
          shownDucking.progress,
        )
      : null,
  ];
}
