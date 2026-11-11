// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, type Ref } from "react";

import { t } from "../../i18n/i18n";

import { LicencesPane } from "./licences/licences-pane";
import { GeneralPane } from "./panes/general-pane";
import { MomentsPane } from "./panes/moments-pane";
import { RecordingPane } from "./panes/recording-pane";
import { SettingsPane } from "./panes/settings-pane";
import { ToolsPane, type ToolsPaneProps } from "./panes/tools-pane";
import { TranscriptionSettingsPanel } from "./sections/transcription-settings";
import { ShortcutsPane } from "./shortcuts/shortcuts-pane";

import type { LicencesControls } from "./licences/use-licences";
import type { MomentSettingsState } from "./sections/use-moment-settings";
import type { TranscriptionControls } from "./sections/use-transcription";
import type { SettingsSection } from "./settings-sections";
import type { GeneralSettings } from "../../bindings/GeneralSettings";
import type { ShortcutAction } from "../../bindings/ShortcutAction";
import type { ShortcutDefaults } from "../../bindings/ShortcutDefaults";
import type { ShortcutSettings } from "../../bindings/ShortcutSettings";

export type SettingsPanesProps = Omit<ToolsPaneProps, "defaults"> & {
  general: GeneralSettings | null;
  licences: LicencesControls;
  moments: MomentSettingsState;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChangeBinding: (action: ShortcutAction, shortcut: string | null) => void;
  onChangeGeneral: (settings: GeneralSettings) => void;
  onChangeReplaceSystemKeys: (replace: boolean) => void;
  onError: (message: string) => void;
  /** Opens the Shortcuts pane on the row of the shortcut with this title. */
  onOpenShortcut: (title: string) => void;
  savingGeneral: boolean;
  savingShortcut: ShortcutAction | null;
  savingSystemKeys: boolean;
  section: SettingsSection;
  shortcuts: ShortcutSettings | null;
  transcription: TranscriptionControls;
  defaults?: ShortcutDefaults | null;
  paneRef?: Ref<HTMLElement>;
  /** The title of the row a search result is bringing into view. */
  revealing?: string | null;
  updateSetting?: ReactNode;
};

/** The pane the sidebar selection asks for. The window owns the shell;
 * each pane its picture and the scrolling under it. */
export function SettingsPanes(props: SettingsPanesProps) {
  const { general, paneRef, section } = props;
  switch (section) {
    case "general":
      return general ? (
        <GeneralPane
          isSaving={props.savingGeneral}
          onChange={props.onChangeGeneral}
          onError={props.onError}
          paneRef={paneRef}
          settings={general}
          updateSetting={props.updateSetting}
        />
      ) : null;
    case "recording":
      return general ? (
        <RecordingPane
          isSaving={props.savingGeneral}
          onChange={props.onChangeGeneral}
          paneRef={paneRef}
          settings={general}
        />
      ) : null;
    case "tools":
      return <ToolsPane {...props} defaults={props.defaults?.glide} />;
    case "moments":
      return (
        <MomentsPane
          moments={props.moments}
          onOpenShortcut={props.onOpenShortcut}
          paneRef={paneRef}
          transcription={props.transcription}
        />
      );
    case "transcription":
      return (
        <SettingsPane
          label={t("settings-section-transcription")}
          paneRef={paneRef}
        >
          <TranscriptionSettingsPanel controls={props.transcription} />
        </SettingsPane>
      );
    case "hotkeys":
      return <ShortcutsPane {...props} />;
    case "licences":
      return <LicencesPane controls={props.licences} paneRef={paneRef} />;
  }
}
