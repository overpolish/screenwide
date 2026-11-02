// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, type Ref } from "react";

import { t } from "../../../i18n/i18n";
import { SettingsPreview } from "../preview/settings-preview";
import { AnnotateSettingsPanel } from "../sections/annotate-settings";
import {
  GlideSettingsPanel,
  type GlideSubject,
} from "../sections/glide-settings";
import { OcrSettingsPanel } from "../sections/ocr-settings";
import { RulerSettingsPanel } from "../sections/ruler-settings";
import { GlidePicture } from "../stage/glide-picture";
import { PreviewArea, PreviewStage } from "../stage/preview-stage";
import { usePreviewSubject } from "../stage/use-preview-subject";

import { SettingsPane } from "./settings-pane";

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";
import type { GlideSettings } from "../../../bindings/GlideSettings";
import type { OcrSettings } from "../../../bindings/OcrSettings";
import type { RulerSettings } from "../../../bindings/RulerSettings";
import type { SettingsPreviewScene } from "../preview/scenes/preview-scenes";

// Tools: each tool in a box of its own, what it does then how it behaves,
// and above them the tool at work, drawn by the tool itself where it can
// be: Glide's own island moving a window, the Ruler measuring, Annotate
// drawing in its colour, OCR reading; double-tapping to centre has its own,
// a window centred. The picture follows the row under the pointer; a tool
// switched off shows its picture greyed and still.

type Subject = GlideSubject | "annotate" | "ocr" | "ruler";

export type ToolsPaneProps = {
  annotate: AnnotateSettings | null;
  glide: GlideSettings | null;
  ocr: OcrSettings | null;
  onChangeAnnotate: (settings: AnnotateSettings) => void;
  onChangeGlide: (settings: GlideSettings) => void;
  onChangeOcr: (settings: OcrSettings) => void;
  onChangeRuler: (settings: RulerSettings) => void;
  ruler: RulerSettings | null;
  savingAnnotate: boolean;
  savingGlide: boolean;
  savingOcr: boolean;
  savingRuler: boolean;
  defaults?: GlideSettings;
  paneRef?: Ref<HTMLElement>;
};

function ToolPicture({
  isOn,
  scene,
}: {
  isOn: boolean;
  scene: SettingsPreviewScene;
}) {
  return (
    <SettingsPreview
      className="absolute inset-x-0 top-0 rounded-panel"
      isOn={isOn}
      scene={scene}
    />
  );
}

export function ToolsPane(props: ToolsPaneProps) {
  const { annotate, glide, ocr, paneRef, ruler } = props;
  const { follow, subject } = usePreviewSubject<Subject>("glide");

  const pictures: Record<Subject, ReactNode> = {
    annotate: annotate && (
      <ToolPicture
        isOn={annotate.enabled}
        scene={{
          color: annotate.defaultColor,
          counterAngle: annotate.defaultCounterAngle,
          counterSize: annotate.defaultCounterSize,
          keep: annotate.keepAnnotationsBetweenSessions,
          kind: "annotate",
        }}
      />
    ),
    glide: glide && (
      <GlidePicture
        cursorFollows={glide.cursorFollows}
        gap={glide.windowGap}
        gesture="side"
        isOn={glide.enabled}
      />
    ),
    glideCenter: glide && (
      <GlidePicture
        cursorFollows={glide.cursorFollows}
        gap={glide.windowGap}
        gesture="center"
        isOn={glide.enabled && glide.doubleTapCenter}
      />
    ),
    ocr: ocr && (
      <ToolPicture isOn={ocr.enabled} scene={{ kind: "textRecognition" }} />
    ),
    ruler: ruler && (
      <ToolPicture isOn={ruler.enabled} scene={{ kind: "ruler" }} />
    ),
  };
  const picture = pictures[subject];

  return (
    <SettingsPane
      label={t("settings-section-tools")}
      paneRef={paneRef}
      stage={
        picture ? (
          <PreviewStage subject={subject}>{picture}</PreviewStage>
        ) : null
      }
    >
      {glide ? (
        <GlideSettingsPanel
          defaults={props.defaults}
          follow={follow}
          isSaving={props.savingGlide}
          onChange={props.onChangeGlide}
          settings={glide}
        />
      ) : null}
      {ruler ? (
        <PreviewArea follow={follow("ruler")}>
          <RulerSettingsPanel
            isSaving={props.savingRuler}
            onChange={props.onChangeRuler}
            settings={ruler}
          />
        </PreviewArea>
      ) : null}
      {annotate ? (
        <PreviewArea follow={follow("annotate")}>
          <AnnotateSettingsPanel
            isSaving={props.savingAnnotate}
            onChange={props.onChangeAnnotate}
            settings={annotate}
          />
        </PreviewArea>
      ) : null}
      {ocr ? (
        <PreviewArea follow={follow("ocr")}>
          <OcrSettingsPanel
            isSaving={props.savingOcr}
            onChange={props.onChangeOcr}
            settings={ocr}
          />
        </PreviewArea>
      ) : null}
    </SettingsPane>
  );
}
