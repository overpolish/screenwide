// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type ComponentProps, useRef, useState } from "react";

import { SkinContext } from "../../../../primitives/skin/skin";
import { hidePopupPanel } from "../../../popup-panel/api";
import { usePopupPanelStore } from "../../../popup-panel/store";
import { addCue } from "../cue-api";

import { CueMoment } from "./cue-moment";
import { CueSources } from "./cue-sources";
import { useCueMoment } from "./use-cue-moment";
import { useCueSources, useCueStills } from "./use-cue-sources";
import { usePickerSize } from "./use-picker-size";

import type { TeleprompterCueTiming } from "../../../../bindings/TeleprompterCueTiming";
import type { ProjectSummary } from "../../../project-browser/types";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

// The teleprompter's cue picker, in its own panel window: the recordings and
// screenshots a cue can show, newest first. A screenshot makes the cue at
// once; a recording opens on its moments, to scrub to the one the cue shows
// and confirm it. The cue is made in the app, which tells the teleprompter,
// and the panel goes.
export function CuePicker({
  at,
  panel,
  script,
}: {
  at: number;
  panel: string;
  script: string;
}) {
  const projects = useCueSources();
  const stills = useCueStills(projects);
  const [recording, setRecording] = useState<ProjectSummary | null>(null);
  const [isMaking, setIsMaking] = useState(false);
  const [failed, setFailed] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);
  usePickerSize(contentRef, recording !== null);
  const close = () => {
    usePopupPanelStore.getState().close(panel);
    hidePopupPanel(true, panel).catch(report("Could not close the cues"));
  };
  const make = (source: string, timing: TeleprompterCueTiming | null) => {
    setIsMaking(true);
    setFailed(false);
    addCue({ at, script, source, timing })
      .then(close)
      .catch((error: unknown) => {
        report("Could not make the cue")(error);
        setIsMaking(false);
        setFailed(true);
      });
  };
  // In the island's skin, as the teleprompter it opens from: round
  // controls, and a search field ringed only when reached by keyboard.
  return (
    <SkinContext value="island">
      <div className={recording ? undefined : "size-full"} ref={contentRef}>
        {recording ? (
          <RecordingMoments
            hasFailed={failed}
            isMaking={isMaking}
            onBack={() => {
              setRecording(null);
              setFailed(false);
            }}
            onConfirm={(timing) => {
              make(recording.file, timing);
            }}
            project={recording}
          />
        ) : (
          <CueSources
            hasFailed={failed}
            isMaking={isMaking}
            onChoose={(project) => {
              if (project.kind === "screenshot") make(project.file, null);
              else setRecording(project);
            }}
            projects={projects}
            stills={stills}
          />
        )}
      </div>
    </SkinContext>
  );
}

function RecordingMoments({
  project,
  ...props
}: Pick<
  ComponentProps<typeof CueMoment>,
  "hasFailed" | "isMaking" | "onBack" | "onConfirm"
> & { project: ProjectSummary }) {
  const { drawn, map, played, reading, setPlayed, strip } =
    useCueMoment(project);
  return (
    <CueMoment
      {...props}
      beats={(reading?.beats ?? []).map(({ beat, span }) => ({
        ...span,
        color: beat.kind === "moment" ? beat.color : undefined,
      }))}
      drawn={drawn}
      map={map}
      onPlayedChange={setPlayed}
      played={played}
      strip={strip}
      title={project.title}
    />
  );
}
