// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, Mic, Monitor, Volume2 } from "lucide-react";

import {
  RecordingOutputSettings,
  recordingVideoTrackOrder,
} from "../screenshot-output";
import {
  ExportArtifact,
  recordingAudioTrackId,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";

type RecordingArtifact = Extract<ExportArtifact, { kind: "recording" }>;

export const recordingTrackTabs = (
  artifact: RecordingArtifact,
  recordingOutput?: RecordingOutputSettings,
) => {
  const tabs: {
    icon: React.ReactNode;
    id: RecordingTrackId;
    label: string;
  }[] = [];
  const videoTabs: typeof tabs = [];
  if (artifact.primaryKind !== "audio") {
    const isCamera = artifact.primaryKind === "camera";
    videoTabs.push({
      icon: isCamera ? <Camera size={15} /> : <Monitor size={15} />,
      id: "primary",
      label: isCamera ? "Camera" : "Screen",
    });
  }
  if (artifact.camera) {
    videoTabs.push({
      icon: <Camera size={15} />,
      id: "camera",
      label: "Camera",
    });
  }
  const videoOrder = recordingOutput
    ? recordingVideoTrackOrder(recordingOutput)
    : (["camera", "primary"] as const);
  tabs.push(
    ...videoTabs.sort(
      (left, right) =>
        videoOrder.indexOf(left.id as RecordingVideoTrackId) -
        videoOrder.indexOf(right.id as RecordingVideoTrackId),
    ),
    ...artifact.audioTracks.map((track) => ({
      icon:
        track.kind === "microphone" ? <Mic size={15} /> : <Volume2 size={15} />,
      id: recordingAudioTrackId(track.streamIndex),
      label: track.label,
    })),
  );
  return tabs;
};
