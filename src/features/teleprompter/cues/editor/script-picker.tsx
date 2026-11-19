// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useState } from "react";

import { SkinContext } from "../../../../primitives/skin/skin";
import { hidePopupPanel } from "../../../popup-panel/api";
import { usePopupPanelStore } from "../../../popup-panel/store";
import { type CueRequest, placeCueInScript } from "../cue-api";
import { useCueSources } from "../picker/use-cue-sources";

import { ScriptChoice } from "./script-choice";

import type { ProjectSummary } from "../../../project-browser/types";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

const isScript = ({ available, kind }: ProjectSummary) =>
  available && kind === "script";

// Which script a cue made in an editor is for, asked in the editor's panel
// window when no teleprompter is open: the script chosen opens in the
// teleprompter, waiting for the cue's word to be picked, and the panel goes.
export function ScriptPicker({
  panel,
  request,
}: {
  panel: string;
  request: CueRequest;
}) {
  const projects = useCueSources(isScript);
  const [isAdding, setIsAdding] = useState(false);
  const [failed, setFailed] = useState(false);
  return (
    <SkinContext value="island">
      <ScriptChoice
        hasFailed={failed}
        isAdding={isAdding}
        onChoose={(script) => {
          setIsAdding(true);
          setFailed(false);
          placeCueInScript({ ...request, script: script.file })
            .then(() => {
              usePopupPanelStore.getState().close(panel);
              return hidePopupPanel(false, panel);
            })
            .catch((error: unknown) => {
              report("Could not add the cue")(error);
              setIsAdding(false);
              setFailed(true);
            });
        }}
        scripts={projects}
      />
    </SkinContext>
  );
}
