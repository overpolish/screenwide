// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard, LayoutGrid, Settings } from "lucide-react";
import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { SidebarNav } from "../../components/base/sidebar-nav/sidebar-nav";
import { Text } from "../../components/base/text/text";
import { WindowHeader } from "../../components/shared/window-header/window-header";

import { GeneralSettingsPanel } from "./general-settings";
import { GlideSettingsPanel } from "./glide-settings";
import { HotkeySettingsPanel } from "./hotkey-settings";
import { useSettingsApi } from "./settings-api-context";
import { LiveSettingsUpdateActions } from "./settings-update-actions";
import {
  GeneralSettings,
  GlideSettings,
  ShortcutAction,
  ShortcutSettings,
} from "./types";

type SettingsSection = "general" | "glide" | "hotkeys";

const sectionTitles: Record<SettingsSection, string> = {
  general: "General",
  glide: "Glide",
  hotkeys: "Shortcuts",
};

export function SettingsWindow({
  updateActions = <LiveSettingsUpdateActions />,
}: {
  updateActions?: ReactNode;
}) {
  const {
    beginShortcutCapture,
    endShortcutCapture,
    getGeneralSettings,
    getGlideSettings,
    getShortcutSettings,
    hideSettings,
    minimize,
    setGeneralSettings,
    setGlideSettings,
    setShortcutBinding,
  } = useSettingsApi();
  const [section, setSection] = useState<SettingsSection>("general");
  const [general, setGeneral] = useState<GeneralSettings | null>(null);
  const [glide, setGlide] = useState<GlideSettings | null>(null);
  const [savingGlide, setSavingGlide] = useState(false);
  const [savingGeneral, setSavingGeneral] = useState(false);
  const [settings, setSettings] = useState<ShortcutSettings | null>(null);
  const [saving, setSaving] = useState<ShortcutAction | null>(null);
  const [error, setError] = useState<string | null>(null);
  const captureQueueRef = useRef(Promise.resolve());
  const captureCountRef = useRef(0);
  const glideSaveRef = useRef<{
    pending: GlideSettings | null;
    running: boolean;
  }>({ pending: null, running: false });
  const onCaptureChange = useCallback(
    (capturing: boolean) => {
      captureCountRef.current += capturing ? 1 : -1;
      // A closing field may still be releasing while the next one prepares.
      if (capturing ? captureCountRef.current > 1 : captureCountRef.current > 0)
        return captureQueueRef.current;
      const operation = captureQueueRef.current
        .catch(() => undefined)
        .then(async () => {
          await (capturing ? beginShortcutCapture() : endShortcutCapture());
        });
      captureQueueRef.current = operation;
      return operation;
    },
    [beginShortcutCapture, endShortcutCapture],
  );

  useEffect(() => {
    Promise.all([
      getGeneralSettings(),
      getGlideSettings(),
      getShortcutSettings(),
    ])
      .then(([generalSettings, glideSettings, shortcutSettings]) => {
        setGeneral(generalSettings);
        setGlide(glideSettings);
        setSettings(shortcutSettings);
      })
      .catch((reason: unknown) => {
        setError(String(reason));
      });
  }, [getGeneralSettings, getGlideSettings, getShortcutSettings]);

  const changeBinding = useCallback(
    (action: ShortcutAction, shortcut: string | null) => {
      setSaving(action);
      setError(null);
      setShortcutBinding(action, shortcut)
        .then(setSettings)
        .catch((reason: unknown) => {
          setError(String(reason));
        })
        .finally(() => {
          setSaving(null);
        });
    },
    [setShortcutBinding],
  );

  const changeGeneral = useCallback(
    (next: GeneralSettings) => {
      setGeneral(next);
      setSavingGeneral(true);
      setError(null);
      setGeneralSettings(next)
        .then(setGeneral)
        .catch((reason: unknown) => {
          setError(String(reason));
          void getGeneralSettings().then(setGeneral);
        })
        .finally(() => {
          setSavingGeneral(false);
        });
    },
    [getGeneralSettings, setGeneralSettings],
  );

  const changeGlide = useCallback(
    (next: GlideSettings) => {
      setGlide(next);
      setError(null);
      const queue = glideSaveRef.current;
      queue.pending = next;
      if (queue.running) return;
      queue.running = true;
      setSavingGlide(true);
      // Keep pointer-rate slider edits live; persist in order and coalesce drafts.
      const save = async () => {
        const hasPending = () => glideSaveRef.current.pending !== null;
        try {
          while (queue.pending) {
            const draft = queue.pending;
            queue.pending = null;
            try {
              const saved = await setGlideSettings(draft);
              if (!hasPending()) setGlide(saved);
            } catch (reason: unknown) {
              setError(String(reason));
              if (!hasPending()) {
                const saved = await getGlideSettings();
                if (!hasPending()) setGlide(saved);
              }
            }
          }
        } catch (reason: unknown) {
          setError(String(reason));
        } finally {
          queue.running = false;
          setSavingGlide(false);
        }
      };
      void save();
    },
    [getGlideSettings, setGlideSettings],
  );

  return (
    <main className="window-surface gap-section flex h-full w-full flex-col overflow-hidden rounded-window text-content-fg">
      <WindowHeader
        actions={updateActions}
        leadingSection={
          <img
            alt="Screenwide"
            className="brightness-0 dark:invert"
            draggable={false}
            src={logoUrl}
          />
        }
        onClose={() => void hideSettings()}
        onMinimize={() => void minimize()}
        title="Settings"
      />
      <div className="gap-layout px-window-inset pb-window-inset flex min-h-0 grow">
        <SidebarNav
          aria-label="Settings sections"
          isExpandable={false}
          items={[
            { icon: <Settings />, id: "general", label: "General" },
            { icon: <LayoutGrid />, id: "glide", label: "Glide" },
            { icon: <Keyboard />, id: "hotkeys", label: "Shortcuts" },
          ]}
          onSelectionChange={(id) => {
            setSection(id as SettingsSection);
          }}
          selected={section}
        />
        <div className="gap-section flex min-h-0 min-w-0 grow flex-col">
          <header className="w-full shrink-0">
            <h1 className="m-0 text-lg font-semibold">
              {sectionTitles[section]}
            </h1>
          </header>
          <section
            aria-label={sectionTitles[section]}
            className="gap-section flex min-h-0 min-w-0 grow flex-col"
          >
            <ScrollArea
              edgeEffect="inset"
              key={section}
              rootClassName="min-h-0 grow"
              scrollbarAutoHide="never"
            >
              {section === "general" && general ? (
                <GeneralSettingsPanel
                  isSaving={savingGeneral}
                  onChange={changeGeneral}
                  onError={setError}
                  settings={general}
                />
              ) : null}
              {section === "glide" && glide ? (
                <GlideSettingsPanel
                  isSaving={savingGlide}
                  onCaptureChange={onCaptureChange}
                  onChange={changeGlide}
                  settings={glide}
                />
              ) : null}
              {section === "hotkeys" ? (
                <HotkeySettingsPanel
                  onCaptureChange={onCaptureChange}
                  onChange={changeBinding}
                  saving={saving}
                  settings={settings}
                />
              ) : null}
            </ScrollArea>
            {error ? (
              <Text className="text-error" role="alert" variant="help">
                {error}
              </Text>
            ) : null}
          </section>
        </div>
      </div>
    </main>
  );
}
