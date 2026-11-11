// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Camera,
  Coffee,
  Flag,
  type LucideIcon,
  PanelsTopLeft,
  PenLine,
  Ruler,
  ScanText,
  Video,
} from "lucide-react";

import { t } from "../../../i18n/i18n";
import {
  modifierDescription,
  modifierKeys,
  modifierTitle,
} from "../sections/glide-rows";
import {
  type GlobalShortcut,
  keepAwakeShortcuts,
  recordingShortcuts,
  screenshotShortcuts,
} from "../sections/global-shortcuts";
import { ocrShortcuts } from "../sections/ocr-shortcuts";
import { rulerShortcuts } from "../sections/ruler-shortcuts";

import { systemKeyEntries } from "./system-key-entries";

import type { ConflictSubject } from "./shortcut-conflicts";
import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";
import type { GlideSettings } from "../../../bindings/GlideSettings";
import type { OcrSettings } from "../../../bindings/OcrSettings";
import type { RulerSettings } from "../../../bindings/RulerSettings";
import type { ShortcutAction } from "../../../bindings/ShortcutAction";
import type { ShortcutDefaults } from "../../../bindings/ShortcutDefaults";
import type { ShortcutSettings } from "../../../bindings/ShortcutSettings";
import type { HotkeyCaptureMode } from "../../../lib/hotkey";
import type { MomentSettingsState } from "../sections/use-moment-settings";

// Every shortcut the app has, from wherever its setting is kept, as one
// list the keyboard map, the rows and the clash check share. Groups follow
// the sidebar: what a feature's key opens comes first in its group, then
// the keys that work while it is open.

export type ShortcutEntry = ConflictSubject & {
  captureMode: HotkeyCaptureMode;
  group: string;
  icon: LucideIcon;
  isClearable: boolean;
  isDisabled: boolean;
  /** Its feature is off, so the key does nothing for now. */
  isOff: boolean;
  onChange: (value: string | null) => void;
  defaultValue?: string | null;
  description?: string;
  /** One of the system's keys, taken for the tool it names. */
  isSystem?: boolean;
};

export type ShortcutSources = {
  annotate: AnnotateSettings | null;
  glide: GlideSettings | null;
  moments: MomentSettingsState;
  ocr: OcrSettings | null;
  onChangeBinding: (action: ShortcutAction, shortcut: string | null) => void;
  onChangeGlide: (settings: GlideSettings) => void;
  onChangeOcr: (settings: OcrSettings) => void;
  onChangeReplaceSystemKeys: (replace: boolean) => void;
  onChangeRuler: (settings: RulerSettings) => void;
  ruler: RulerSettings | null;
  savingAnnotate: boolean;
  savingGlide: boolean;
  savingOcr: boolean;
  savingRuler: boolean;
  savingShortcut: ShortcutAction | null;
  savingSystemKeys: boolean;
  shortcuts: ShortcutSettings | null;
  /** Includes the system's keys Screenwide can take on this platform. */
  defaults?: ShortcutDefaults | null;
};

const bindingOf = (
  settings: ShortcutSettings | null | undefined,
  action: ShortcutAction,
) =>
  settings?.bindings.find((binding) => binding.action === action)?.shortcut ??
  null;

export function shortcutEntries(sources: ShortcutSources): ShortcutEntry[] {
  const { annotate, defaults, glide, moments, ocr, ruler, shortcuts } = sources;
  const savingGlobal = !shortcuts || sources.savingShortcut !== null;
  const global = ({
    action,
    description,
    group,
    icon,
    isOff = false,
    label,
  }: GlobalShortcut & {
    group: string;
    icon: LucideIcon;
    isOff?: boolean;
  }): ShortcutEntry => ({
    captureMode: "shortcut",
    defaultValue: bindingOf(defaults?.shortcuts, action),
    description,
    group,
    icon,
    id: action,
    isClearable: true,
    isDisabled: savingGlobal || isOff,
    isOff,
    kind: "combo",
    label,
    onChange: (value) => {
      sources.onChangeBinding(action, value);
    },
    scope: "anywhere",
    value: bindingOf(shortcuts, action),
  });

  const recording = t("settings-recording");
  const screenshots = t("settings-screenshots");
  const entries: ShortcutEntry[] = [
    ...recordingShortcuts().map((item) =>
      global({ ...item, group: recording, icon: Video }),
    ),
    ...screenshotShortcuts().map((item) =>
      global({ ...item, group: screenshots, icon: Camera }),
    ),
  ];

  const momentSettings = moments.settings;
  for (const kind of momentSettings?.kinds ?? [])
    entries.push({
      captureMode: "shortcut",
      group: t("settings-section-moments"),
      icon: Flag,
      id: `moment ${kind.id}`,
      isClearable: true,
      isDisabled: moments.isSaving,
      isOff: false,
      kind: "combo",
      label: t("settings-shortcut-place-moment", { moment: kind.name }),
      onChange: (shortcut) => {
        if (!momentSettings) return;
        void moments.save({
          ...momentSettings,
          kinds: momentSettings.kinds.map((other) =>
            other.id === kind.id ? { ...other, shortcut } : other,
          ),
        });
      },
      scope: "anywhere",
      value: kind.shortcut,
    });

  if (glide) {
    const isOff = !glide.enabled;
    for (const key of modifierKeys)
      entries.push({
        captureMode: "single-control",
        defaultValue: defaults?.glide[key],
        description: modifierDescription(key),
        group: t("settings-section-glide"),
        icon: PanelsTopLeft,
        id: `glide ${key}`,
        isClearable: false,
        isDisabled: isOff || sources.savingGlide,
        isOff,
        kind: "hold",
        label: modifierTitle(key),
        // Taking a key another one has swaps the two, so no two share one.
        onChange: (value) => {
          if (value === null) return;
          const taken = modifierKeys.find(
            (other) => other !== key && glide[other] === value,
          );
          sources.onChangeGlide({
            ...glide,
            [key]: value,
            ...(taken ? { [taken]: glide[key] } : {}),
          });
        },
        scope: "anywhere",
        value: glide[key],
      });
  }

  const rulerGroup = t("settings-section-ruler");
  const rulerOff = !ruler?.enabled;
  entries.push(
    global({
      action: "rulerOverlay",
      group: rulerGroup,
      icon: Ruler,
      isOff: rulerOff,
      label: t("settings-ruler-show"),
    }),
  );
  for (const { action, description, label } of rulerShortcuts())
    entries.push({
      captureMode: "local-shortcut",
      defaultValue: defaults?.ruler.bindings[action],
      description,
      group: rulerGroup,
      icon: Ruler,
      id: `ruler ${action}`,
      isClearable: true,
      isDisabled: !ruler || rulerOff || sources.savingRuler,
      isOff: rulerOff,
      kind: "combo",
      label,
      onChange: (value) => {
        if (ruler)
          sources.onChangeRuler({
            ...ruler,
            bindings: { ...ruler.bindings, [action]: value },
          });
      },
      scope: "ruler",
      value: ruler?.bindings[action] ?? null,
    });

  const annotateGroup = t("settings-section-annotate");
  const annotateOff = !annotate?.enabled;
  for (const item of [
    { action: "annotateOverlay", label: t("settings-annotate-draw") },
    { action: "annotateClear", label: t("settings-annotate-clear") },
  ] satisfies GlobalShortcut[]) {
    const entry = global({
      ...item,
      group: annotateGroup,
      icon: PenLine,
      isOff: annotateOff,
    });
    entries.push({
      ...entry,
      isDisabled: entry.isDisabled || sources.savingAnnotate,
    });
  }

  const ocrGroup = t("settings-section-ocr");
  const ocrOff = !ocr?.enabled;
  entries.push(
    global({
      action: "recognizeText",
      group: ocrGroup,
      icon: ScanText,
      isOff: ocrOff,
      label: t("settings-ocr-activate"),
    }),
  );
  for (const { action, description, label } of ocrShortcuts())
    entries.push({
      captureMode: "local-shortcut",
      defaultValue: defaults?.ocr.bindings[action],
      description,
      group: ocrGroup,
      icon: ScanText,
      id: `ocr ${action}`,
      isClearable: true,
      isDisabled: !ocr || ocrOff || sources.savingOcr,
      isOff: ocrOff,
      kind: "combo",
      label,
      onChange: (value) => {
        if (ocr)
          sources.onChangeOcr({
            ...ocr,
            bindings: { ...ocr.bindings, [action]: value },
          });
      },
      scope: "ocr",
      value: ocr?.bindings[action] ?? null,
    });

  const keepAwake = t("settings-keep-awake");
  entries.push(
    ...keepAwakeShortcuts().map((item) =>
      global({ ...item, group: keepAwake, icon: Coffee }),
    ),
  );

  return [...entries, ...systemKeyEntries(entries, sources)];
}
