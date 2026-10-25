// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Camera,
  Flag,
  PanelsTopLeft,
  PenLine,
  Ruler,
  ScanText,
  Video,
} from "lucide-react";

import {
  modifierDescription,
  modifierKeys,
  modifierTitle,
} from "../../../../features/settings/sections/glide-rows";
import {
  recordingShortcuts,
  screenshotShortcuts,
} from "../../../../features/settings/sections/global-shortcuts";
import { ocrShortcuts } from "../../../../features/settings/sections/ocr-shortcuts";
import { rulerShortcuts } from "../../../../features/settings/sections/ruler-shortcuts";

import type { ShortcutEntry } from "./shortcut-model";
import type { ShortcutAction } from "../../../../bindings/ShortcutAction";
import type { ExploredSettings } from "../settings-state";

// Every shortcut the app has, from wherever its setting lives, as one list
// the map and the rows share. A tool's own keys work only while it is open;
// the key that opens it works anywhere, so it is listed with the others
// that work anywhere, under the tool's name.

export function shortcutEntries(settings: ExploredSettings): ShortcutEntry[] {
  const { annotate, glide, moments, ocr, ruler, shortcuts } = settings;
  const global = ({
    action,
    ...entry
  }: Pick<
    ShortcutEntry,
    "description" | "group" | "icon" | "isOff" | "label"
  > & {
    action: ShortcutAction;
  }): ShortcutEntry => ({
    ...entry,
    captureMode: "shortcut",
    id: action,
    kind: "combo",
    onChange: (value) => {
      settings.setBinding(action, value);
    },
    scope: "anywhere",
    value:
      shortcuts?.bindings.find((binding) => binding.action === action)
        ?.shortcut ?? null,
  });

  const entries: ShortcutEntry[] = [
    ...recordingShortcuts().map((shortcut) =>
      global({ ...shortcut, group: "Recording", icon: Video }),
    ),
    ...screenshotShortcuts().map((shortcut) =>
      global({ ...shortcut, group: "Screenshots", icon: Camera }),
    ),
  ];

  const kinds = moments.settings?.kinds ?? [];
  for (const kind of kinds)
    entries.push({
      captureMode: "shortcut",
      description: "Hold it to record a voice note.",
      group: "Moments",
      icon: Flag,
      id: `moment ${kind.id}`,
      kind: "combo",
      label: `Place ${kind.name}`,
      onChange: (shortcut) => {
        if (!moments.settings) return;
        void moments.save({
          ...moments.settings,
          kinds: kinds.map((other) =>
            other.id === kind.id ? { ...other, shortcut } : other,
          ),
        });
      },
      scope: "anywhere",
      value: kind.shortcut,
    });

  if (glide)
    for (const key of modifierKeys)
      entries.push({
        captureMode: "single-control",
        description: modifierDescription(key),
        group: "Glide",
        icon: PanelsTopLeft,
        id: `glide ${key}`,
        isOff: !glide.enabled,
        kind: "hold",
        label: modifierTitle(key),
        // Taking a key another one has swaps the two, so no two share one.
        onChange: (value) => {
          if (value === null) return;
          const taken = modifierKeys.find(
            (other) => other !== key && glide[other] === value,
          );
          settings.setGlide({
            ...glide,
            [key]: value,
            ...(taken ? { [taken]: glide[key] } : {}),
          });
        },
        scope: "anywhere",
        value: glide[key],
      });

  entries.push(
    global({
      action: "rulerOverlay",
      group: "Ruler",
      icon: Ruler,
      isOff: !ruler?.enabled,
      label: "Show ruler",
    }),
    global({
      action: "annotateOverlay",
      group: "Annotate",
      icon: PenLine,
      isOff: !annotate?.enabled,
      label: "Draw on the screen",
    }),
    global({
      action: "annotateClear",
      group: "Annotate",
      icon: PenLine,
      isOff: !annotate?.enabled,
      label: "Clear annotations",
    }),
    global({
      action: "recognizeText",
      group: "OCR",
      icon: ScanText,
      isOff: !ocr?.enabled,
      label: "Read text or a QR code",
    }),
  );

  if (ruler)
    for (const { action, description, label } of rulerShortcuts())
      entries.push({
        captureMode: "local-shortcut",
        description,
        group: "While the ruler is open",
        icon: Ruler,
        id: `ruler ${action}`,
        isOff: !ruler.enabled,
        kind: "combo",
        label,
        onChange: (value) => {
          settings.setRuler({
            ...ruler,
            bindings: { ...ruler.bindings, [action]: value },
          });
        },
        scope: "ruler",
        value: ruler.bindings[action] ?? null,
      });

  if (ocr)
    for (const { action, description, label } of ocrShortcuts())
      entries.push({
        captureMode: "local-shortcut",
        description,
        group: "While text is showing",
        icon: ScanText,
        id: `ocr ${action}`,
        isOff: !ocr.enabled,
        kind: "combo",
        label,
        onChange: (value) => {
          settings.setOcr({
            ...ocr,
            bindings: { ...ocr.bindings, [action]: value },
          });
        },
        scope: "ocr",
        value: ocr.bindings[action] ?? null,
      });

  return entries;
}
