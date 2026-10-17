// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { CircularProgress } from "../../../../components/base/circular-progress/circular-progress";
import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { CaptionsPatch } from "../../recording/captions/recording-captions";
import { EditorKind } from "../../types";
import { TranscriptSetup } from "../transcript/transcript-setup";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { CaptionEditor } from "./caption-editor";
import { CaptionLayoutTab } from "./caption-layout-tab";
import { CaptionPanelTab, useCaptionPanelTab } from "./caption-panel-tab";
import { CaptionStyleTab } from "./caption-style-tab";
import { CaptionWordsRows } from "./caption-words-rows";

import type { TextLayout } from "../../../../bindings/TextLayout";

/** Captions with no layout option set by hand, as one object. */
const NO_LAYOUT: TextLayout = {};

const tabs = (): { id: CaptionPanelTab; label: string }[] => [
  { id: "words", label: t("editor-panels-captions-tab-words") },
  { id: "style", label: t("editor-panels-captions-tab-style") },
  { id: "layout", label: t("editor-panels-captions-tab-layout") },
];

/**
 * The Captions tool's own controls: whether they show, the caption text,
 * then three tabs: Words for what the captions say, Style for how they
 * look, and Layout for how large they are and how much each holds.
 * Everything here is the whole recording's and part of its edit, so it
 * undoes with the rest; each tab can also keep what it sets for new
 * captions. Until there is a transcript, the panel offers to make one,
 * since the captions are made of its words.
 */
export function CaptionsPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const [tab, setTab] = useCaptionPanelTab();
  const { captions, isLocked, textTemplates: templates, transcript } = snapshot;
  // The panel window is shown once it has something to measure, so it shows
  // the panel loading rather than waiting, unseen, for the editor's snapshot.
  if (!captions || !transcript || !templates)
    return (
      <div className="flex items-center justify-center py-layout">
        <CircularProgress
          aria-label={t("editor-panels-captions-loading")}
          isIndeterminate
        />
      </div>
    );

  if (
    transcript.model?.status !== "downloaded" ||
    transcript.status !== "ready"
  )
    return (
      <TranscriptSetup
        description={t("editor-panels-captions-description")}
        isLocked={isLocked}
        onCancel={() => {
          change({ transcriptRequest: { type: "cancel" } });
        }}
        onDownload={() => {
          change({ transcriptRequest: { type: "download" } });
        }}
        onTranscribe={() => {
          // Transcribing from here is asking for captions, so they come on
          // and show as soon as the words arrive.
          change({
            captions: { enabled: true },
            transcriptRequest: { type: "transcribe" },
          });
        }}
        transcript={transcript}
      />
    );

  const ask = (patch: CaptionsPatch) => {
    change({ captions: patch });
  };
  const { settings, style } = captions;
  const layout = captions.layout ?? NO_LAYOUT;
  const isDisabled = isLocked || !captions.enabled;

  return (
    <div className="flex flex-col gap-section">
      <ControlRow title={t("editor-panels-captions-show")}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isLocked}
            isSelected={captions.enabled}
            onChange={(enabled) => {
              ask({ enabled });
            }}
          />
        )}
      </ControlRow>

      {/* Always open, captions on or off, so the text can be put right
       * before they are shown. */}
      <CaptionEditor
        captions={captions}
        isDisabled={isLocked}
        onChange={ask}
        onCorrect={(correction) => {
          change({ transcriptRequest: { correction, type: "correct" } });
        }}
        onSeek={(ms) => {
          change({ transcriptRequest: { ms, type: "seek" } });
        }}
        transcript={transcript}
      />

      <PillGroup
        aria-label={t("editor-panels-captions-tabs")}
        className="w-full"
        display="label"
        itemClassName="flex-1"
        items={tabs()}
        onSelectionChange={(id) => {
          setTab(id as CaptionPanelTab);
        }}
        selected={tab}
      />

      {tab === "words" ? (
        <CaptionWordsRows
          defaults={templates.defaultSettings}
          isDisabled={isDisabled}
          onDefaultsChange={(next) => {
            change({ captionDefaults: { settings: next } });
          }}
          onSettingsChange={(next) => {
            ask({ settings: next });
          }}
          settings={settings}
        />
      ) : tab === "style" ? (
        <CaptionStyleTab
          isDisabled={isDisabled}
          layout={layout}
          onChange={(next) => {
            ask({ style: next });
          }}
          onDefaultsChange={(captionDefaults) => {
            change({ captionDefaults });
          }}
          savedColors={{
            colors: snapshot.annotationColors,
            onRemove: (color) => {
              change({ removeAnnotationColor: color });
            },
            onSave: (color) => {
              change({ saveAnnotationColor: color });
            },
          }}
          style={style}
          templates={templates}
        />
      ) : (
        <CaptionLayoutTab
          isDisabled={isDisabled}
          layout={layout}
          onChange={ask}
          onDefaultsChange={(next) => {
            change({ captionDefaults: { layout: next } });
          }}
          style={style}
          templates={templates}
        />
      )}

      <div className="flex justify-end gap-control">
        <Button
          isDisabled={isLocked || !captions.hidden?.length}
          onPress={() => {
            ask({ hidden: [] });
          }}
        >
          {t("editor-panels-captions-restore")}
        </Button>
      </div>
    </div>
  );
}
