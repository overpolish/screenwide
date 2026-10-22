// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { withThemeByClassName } from "@storybook/addon-themes";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { createElement } from "react";
import { I18nProvider } from "react-aria-components";
import { themes } from "storybook/theming";

import { sampleCaptions } from "../src/features/editor/tool-panels/captions/caption-text-fixtures";
import { sampleTranscriptWords } from "../src/features/editor/tool-panels/transcript/transcript-panel-fixtures";
import {
  appDirection,
  applyDirection,
  appLocale,
  LANGUAGES,
  layoutLocale,
  loadAppLocale,
  loadLocale,
  PSEUDO,
  PSEUDO_BIDI,
} from "../src/i18n/i18n";
import { installKeyboardNavigationModality } from "../src/lib/keyboard-navigation-modality";
import { isWindowsHost, stampPlatform } from "../src/lib/platform";
import { synchronizeSystemAccent } from "../src/lib/system-accent";

import { DocsPage } from "./docs-page";

import type { Decorator, Loader, Preview } from "@storybook/react-vite";

import "../src/index.css";
import "./styles.css";

const isNativePreview =
  new URLSearchParams(window.location.search).get("screenwide-native") === "1";

// The app stamps `data-platform` in `main.tsx` for what the OS dictates:
// shortcut labels and the traffic-light inset. The look is the same on both,
// so stories take the host's platform and offer no switch.
const hostPlatform = isWindowsHost ? "windows" : "macos";

// The theme decorator applies its class only once the manager has sent the
// globals, so every reload rendered light first and then flipped. The iframe
// URL already carries the chosen theme in `globals`, so the class is applied
// here, before any story renders; the decorator takes over from there.
if (!isNativePreview) {
  const globals = new URLSearchParams(window.location.search).get("globals");
  const theme = /(?:^|;)theme:(\w+)/.exec(globals ?? "")?.[1] ?? "dark";
  document.documentElement.classList.add(theme === "light" ? "light" : "dark");
  stampPlatform(hostPlatform);
}

if (isNativePreview) {
  document.documentElement.classList.add("screenwide-native-preview");
  const systemDarkMode = window.matchMedia("(prefers-color-scheme: dark)");
  const synchronizeNativeTheme = ({
    matches,
  }: MediaQueryListEvent | MediaQueryList) => {
    document.documentElement.classList.toggle("dark", matches);
    document.documentElement.classList.toggle("light", !matches);
  };

  synchronizeNativeTheme(systemDarkMode);
  systemDarkMode.addEventListener("change", synchronizeNativeTheme);
  stampPlatform(hostPlatform);
  // The native preview runs inside the app shell, so the OS accent is real.
  synchronizeSystemAccent();
}

/** A quarter second of silence as a WAV file, so a voice note decodes and
 * its play button runs without sound. */
const silentWav = () => {
  const samples = 2000;
  const view = new DataView(new ArrayBuffer(44 + samples * 2));
  const text = (offset: number, value: string) => {
    for (let index = 0; index < value.length; index += 1)
      view.setUint8(offset + index, value.charCodeAt(index));
  };
  text(0, "RIFF");
  view.setUint32(4, 36 + samples * 2, true);
  text(8, "WAVE");
  text(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, 8000, true); // sample rate
  view.setUint32(28, 16000, true); // byte rate
  view.setUint16(32, 2, true); // block align
  view.setUint16(34, 16, true); // bits per sample
  text(36, "data");
  view.setUint32(40, samples * 2, true);
  return view.buffer;
};

/** The text of each word `find_transcript_marks` was asked about. */
const askedWords = (payload: unknown): string[] =>
  payload &&
  typeof payload === "object" &&
  "words" in payload &&
  Array.isArray(payload.words)
    ? payload.words.map((word: unknown) =>
        word &&
        typeof word === "object" &&
        "text" in word &&
        typeof word.text === "string"
          ? word.text
          : "",
      )
    : [];

/** The fillers and profanity among the words asked about, by the sample
 * transcript's own few, found by their words alone as the lists would. */
const storyTranscriptMarks = (payload: unknown) => {
  const words = askedWords(payload);
  const found = (list: readonly string[]) =>
    words.flatMap((text, index) => {
      const folded = text.toLowerCase().replace(/[^\p{L}']/gu, "");
      return list.includes(folded)
        ? [{ first: index, folded, last: index }]
        : [];
    });
  return { fillers: found(["uh", "um"]), profanity: found(["damn"]) };
};

// Stub the Tauri runtime so stories mounting Tauri-touching components render
// outside the desktop app. This installs `window.__TAURI_INTERNALS__` with
// `invoke`, `transformCallback`, `unregisterCallback`, `runCallback`, and the
// event-plugin handler. Without it, e.g. `new Channel()` throws because
// `transformCallback` is undefined. Run as a module-load side effect so the
// runtime exists before the first story's effects fire. Data queries must keep
// their response shape, even without native content. Unknown commands resolve
// to `null`; these stories render layout rather than live pixels.
if (!isNativePreview) {
  mockIPC(
    (command, payload) => {
      switch (command) {
        // Caption previews are drawn by the native compositor, which a
        // browser story cannot reach; tiles show their names alone.
        case "get_recording_keyboard_timeline":
        case "get_recording_moments":
        case "make_recording_captions":
        case "open_standalone_listboxes":
        case "render_caption_previews":
          return [];
        case "installed_font_families": {
          const every = [100, 200, 300, 400, 500, 600, 700, 800, 900];
          // The app's own families, as the editor lists them, among a few
          // a system has.
          return [
            { italic: false, name: "Anton", weights: [400] },
            { italic: false, name: "DM Sans", weights: every },
            { italic: true, name: "Georgia", weights: [400, 700] },
            {
              italic: true,
              name: "Helvetica Neue",
              weights: [100, 300, 400, 500, 700],
            },
            { italic: true, name: "Instrument Serif", weights: [400] },
            { italic: false, name: "Montserrat", weights: every },
            { italic: true, name: "Times New Roman", weights: [400, 700] },
          ];
        }
        case "find_transcript_marks":
          return storyTranscriptMarks(payload);
        // The sample transcript and the captions made of it, for panels
        // that read them from the editor rather than their snapshot.
        case "get_recording_transcript":
          return {
            language: "en",
            model: "story",
            words: sampleTranscriptWords,
          };
        case "recording_captions":
          return sampleCaptions;
        case "get_recording_moment_note":
          return silentWav();
        default:
          return null;
      }
    },
    { shouldMockEvents: true },
  );
  // `getCurrentWindow()` reads the label of the window this page runs in
  // off the runtime's metadata, which the IPC mock alone does not install.
  // The label is not an editor's, so `currentEditorKind()` stays null and
  // an editor story takes its workspace from the artifact it is given.
  mockWindows("storybook");
}

// Initialize React Aria's focus tracking before Storybook's test loader wraps
// HTMLElement.prototype.focus. Lazy initialization after that wrapper is unsafe.
const [reactAria, storybookComponents] = await Promise.all([
  import("react-aria"),
  import("storybook/internal/components"),
]);

Object.freeze([reactAria.useOverlay, storybookComponents.Button]);

const disposeKeyboardNavigation = installKeyboardNavigationModality();
const previewHot = (
  import.meta as { hot?: { dispose: (cb: () => void) => void } }
).hot;
previewHot?.dispose(disposeKeyboardNavigation);

// Storybook holds each render open until running animations finish, capped at
// 5 s, and runs decorator effects such as the theme class only afterwards.
// OverlayScrollbars moves its handles with scroll-driven animations, which run
// for as long as they exist and never finish, so every story with a
// `ScrollArea` stalled for the full cap. Only clock-driven animations can
// finish, so the preview reports only those.
const nativeGetAnimations = document.getAnimations.bind(document);
document.getAnimations = () =>
  nativeGetAnimations().filter(
    (animation) => animation.timeline instanceof DocumentTimeline,
  );
// Removing the own property restores the prototype's method.
previewHot?.dispose(() => Reflect.deleteProperty(document, "getAnimations"));

const languageNames = new Intl.DisplayNames(["en"], { type: "language" });

const preview: Preview = {
  globalTypes: isNativePreview
    ? {}
    : {
        // Lays any language out right to left, to check mirroring with text
        // that still reads.
        direction: {
          description: "Layout direction",
          toolbar: {
            dynamicTitle: true,
            icon: "transfer",
            items: [
              { title: "Language's direction", value: "auto" },
              { title: "Right to left", value: "rtl" },
            ],
            title: "Direction",
          },
        },
        // Every translation in `locales/`, and the pseudo-locales: one shows
        // text that skips translation and layouts that break under longer
        // words, the other the same in a right-to-left layout.
        locale: {
          description: "Language",
          toolbar: {
            dynamicTitle: true,
            icon: "globe",
            items: [
              ...LANGUAGES.map((language) => ({
                title: languageNames.of(language) ?? language,
                value: language,
              })),
              { title: "Pseudo-locale", value: PSEUDO },
              { title: "Pseudo-locale (right to left)", value: PSEUDO_BIDI },
            ],
            title: "Language",
          },
        },
        // The notch on MacBooks, which the recording controls are laid out
        // around when there is one. Switched by its own toolbar toggle
        // (`notch-tool.tsx`), shown only on stories that set
        // `parameters.notch`.
        notch: { description: "MacBook notch" },
      },
  initialGlobals: isNativePreview
    ? {}
    : { direction: "auto", locale: LANGUAGES[0], notch: "on" },
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    docs: {
      page: DocsPage,
      theme: themes.dark,
    },
    options: {
      // The layers bottom up, as the source builds on them; work in progress
      // after. Alphabetical within each group.
      storySort: {
        method: "alphabetical",
        order: [
          "Foundations",
          "Primitives",
          "Composites",
          "Features",
          "Legacy",
          "Explorations",
        ],
      },
    },
  },
  tags: ["autodocs"],
};

// Loaded before each render, so a story's first frame is in the language.
// The native preview runs in the app shell, so it takes the app's language.
const loadStoryLocale: Loader = async ({ globals }) => {
  if (isNativePreview) {
    await loadAppLocale();
    return {};
  }
  const language =
    typeof globals.locale === "string" ? globals.locale : LANGUAGES[0];
  await loadLocale({ formatLocale: language, language });
  applyDirection(storyDirection(globals));
  return {};
};

const storyDirection = (globals: Record<string, unknown>) =>
  globals.direction === "rtl" ? "rtl" : appDirection();

// Keyed by language and direction so a switch remounts the story and nothing
// keeps text or layout from the previous one.
const withLocale: Decorator = (Story, { globals }) => {
  const { language } = appLocale();
  const direction = storyDirection(globals);
  return createElement(
    I18nProvider,
    { key: `${language}-${direction}`, locale: layoutLocale(direction) },
    Story(),
  );
};

export const loaders = [loadStoryLocale];

export const decorators = (
  isNativePreview
    ? [withLocale]
    : [
        withLocale,
        withThemeByClassName({
          defaultTheme: "dark",
          parentSelector: "html",
          themes: {
            dark: "dark",
            light: "light",
          },
        }),
      ]
) as Decorator[];

export default preview;
