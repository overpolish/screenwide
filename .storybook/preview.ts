// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { withThemeByClassName } from "@storybook/addon-themes";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { createElement } from "react";
import { I18nProvider } from "react-aria-components";
import { themes } from "storybook/theming";

import {
  appLocale,
  LANGUAGES,
  loadAppLocale,
  loadLocale,
  PSEUDO,
} from "../src/i18n/i18n";
import { installKeyboardNavigationModality } from "../src/lib/keyboard-navigation-modality";
import { synchronizeSystemAccent } from "../src/lib/system-accent";

import type { Decorator, Loader, Preview } from "@storybook/react-vite";

import "../src/index.css";
import "./styles.css";

const isNativePreview =
  new URLSearchParams(window.location.search).get("screenwide-native") === "1";

// The skin is a token layer keyed off `data-platform`, as the app sets it in
// `main.tsx`. Stories default to the host OS so a reviewer sees their own
// platform first, and the toolbar switches to the other one. The native
// preview runs inside the real app shell, so it always takes the host.
type Platform = "macos" | "windows";
const hostPlatform: Platform = navigator.userAgent.includes("Windows")
  ? "windows"
  : "macos";

const applyPlatform = (platform: Platform) => {
  if (platform === "windows") {
    document.documentElement.dataset.platform = "windows";
  } else {
    delete document.documentElement.dataset.platform;
  }
};

// The theme decorator applies its class only once the manager has sent the
// globals, so every reload rendered light first and then flipped. The iframe
// URL already carries the chosen theme in `globals`, so the class is applied
// here, before any story renders; the decorator takes over from there.
if (!isNativePreview) {
  const globals = new URLSearchParams(window.location.search).get("globals");
  const theme = /(?:^|;)theme:(\w+)/.exec(globals ?? "")?.[1] ?? "dark";
  document.documentElement.classList.add(theme === "light" ? "light" : "dark");
  const platform = /(?:^|;)platform:(\w+)/.exec(globals ?? "")?.[1];
  applyPlatform(
    platform === "windows" || platform === "macos" ? platform : hostPlatform,
  );
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
  applyPlatform(hostPlatform);
  // The native preview runs inside the app shell, so the OS accent is real.
  synchronizeSystemAccent();
}

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
    (command) => {
      if (command === "get_recording_keyboard_timeline") return [];
      return null;
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
        // Every translation in `locales/`, and the pseudo-locale, which
        // shows text that skips translation and layouts that break under
        // longer words.
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
            ],
            title: "Language",
          },
        },
        platform: {
          description: "Platform skin",
          toolbar: {
            dynamicTitle: true,
            icon: hostPlatform === "windows" ? "windows" : "apple",
            items: [
              { icon: "apple", title: "macOS", value: "macos" },
              { icon: "windows", title: "Windows", value: "windows" },
            ],
            title: "Platform",
          },
        },
      },
  initialGlobals: isNativePreview
    ? {}
    : { locale: LANGUAGES[0], platform: hostPlatform },
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    docs: {
      theme: themes.dark,
    },
    options: {
      storySort: {
        method: "alphabetical",
      },
    },
  },
  tags: ["autodocs"],
};

const withPlatform: Decorator = (Story, { globals }) => {
  applyPlatform(globals.platform === "windows" ? "windows" : "macos");
  return Story();
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
  return {};
};

// Keyed by language so a switch remounts the story and nothing keeps text
// from the previous one.
const withLocale: Decorator = (Story) => {
  const { formatLocale, language } = appLocale();
  return createElement(
    I18nProvider,
    { key: language, locale: formatLocale },
    Story(),
  );
};

export const loaders = [loadStoryLocale];

export const decorators = (
  isNativePreview
    ? [withLocale]
    : [
        withLocale,
        withPlatform,
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
