// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Clone, Copy)]
pub enum WindowLabel {
  /// The recording workspace's window. Each editor workspace has one of its
  /// own so a recording can wait for a decision while a screenshot is edited.
  EditorRecording,
  EditorScreenshot,
  /// Each editor workspace's export options window, a non-movable child of the
  /// editor it belongs to.
  ExportRecording,
  ExportScreenshot,
  /// Each editor workspace's tool panel window, a sticky panel attached to
  /// the editor it belongs to. One per workspace, so a tool in hand in one
  /// editor cannot take the other's panel away.
  ToolPanelRecording,
  ToolPanelScreenshot,
  /// The shared alert that reports a failure, nobody's child.
  Alert,
  /// The shared tooltip, a floating panel that describes whatever is hovered.
  Tooltip,
  #[cfg(target_os = "macos")]
  Permissions,
  /// The live annotation overlay's transparent host surface.
  Annotate,
  /// The live annotation overlay's floating toolbar, above every host.
  AnnotateToolbar,
  Glide,
  RecordingBar,
  RecordingDock,
  Ruler,
  QrDetails,
  /// The project browser: recent projects and the folders that hold them.
  Projects,
  Settings,
  RegionSelector,
  RecordingSourceSelector,
  StandaloneListbox,
  TextRecognition,
  Update,
}

impl WindowLabel {
  pub const ALL: &'static [Self] = &[
    Self::EditorRecording,
    Self::EditorScreenshot,
    Self::ExportRecording,
    Self::ExportScreenshot,
    Self::ToolPanelRecording,
    Self::ToolPanelScreenshot,
    Self::Alert,
    Self::Tooltip,
    #[cfg(target_os = "macos")]
    Self::Permissions,
    Self::Annotate,
    Self::AnnotateToolbar,
    Self::Glide,
    Self::RecordingBar,
    Self::RecordingDock,
    Self::Ruler,
    Self::QrDetails,
    Self::Projects,
    Self::Settings,
    Self::RegionSelector,
    Self::RecordingSourceSelector,
    Self::StandaloneListbox,
    Self::TextRecognition,
    Self::Update,
  ];

  pub const fn as_str(self) -> &'static str {
    match self {
      Self::EditorRecording => "editor-recording",
      Self::EditorScreenshot => "editor-screenshot",
      Self::ExportRecording => "export-recording",
      Self::ExportScreenshot => "export-screenshot",
      Self::ToolPanelRecording => "tool-panel-recording",
      Self::ToolPanelScreenshot => "tool-panel-screenshot",
      Self::Alert => "alert",
      Self::Tooltip => "tooltip",
      #[cfg(target_os = "macos")]
      Self::Permissions => "permissions",
      Self::Annotate => "annotate",
      Self::AnnotateToolbar => "annotate-toolbar",
      Self::Glide => "glide",
      Self::RecordingBar => "recording-bar",
      Self::RecordingDock => "recording-dock",
      Self::Ruler => "ruler",
      Self::QrDetails => "qr-details",
      Self::Projects => "projects",
      Self::Settings => "settings",
      Self::RegionSelector => "region-selector",
      Self::RecordingSourceSelector => "recording-source-selector",
      Self::StandaloneListbox => "standalone-listbox",
      Self::TextRecognition => "text-recognition",
      Self::Update => "update",
    }
  }

  /// The title the operating system shows for the window, in the app's
  /// language: in the Window menu, the task switcher and to assistive
  /// technology. The page draws its own title, so this is never on screen in
  /// the window itself.
  pub fn title(self) -> String {
    use crate::i18n::t;
    match self {
      Self::EditorRecording | Self::EditorScreenshot => t!("window-titles-editor"),
      Self::ExportRecording => t!("window-titles-export-recording"),
      Self::ExportScreenshot => t!("window-titles-export-screenshot"),
      Self::ToolPanelRecording => t!("window-titles-tool-panel-recording"),
      Self::ToolPanelScreenshot => t!("window-titles-tool-panel-screenshot"),
      #[cfg(target_os = "macos")]
      Self::Permissions => t!("window-titles-permissions"),
      Self::AnnotateToolbar => t!("window-titles-annotate-toolbar"),
      Self::Glide => t!("window-titles-glide"),
      Self::RecordingBar => t!("window-titles-recording-bar"),
      Self::RecordingDock => t!("window-titles-recording-dock"),
      Self::QrDetails => t!("window-titles-qr-details"),
      Self::Projects => t!("window-titles-projects"),
      Self::Settings => t!("window-titles-settings"),
      Self::RegionSelector => t!("window-titles-region-selector"),
      Self::RecordingSourceSelector => t!("window-titles-source-selector"),
      Self::StandaloneListbox => t!("window-titles-listbox"),
      Self::Update => t!("window-titles-update"),
      Self::Alert | Self::Tooltip | Self::Annotate | Self::Ruler | Self::TextRecognition => {
        t!("window-titles-app")
      }
    }
  }

  /// The window's entry in `tauri.conf.json`, titled in the app's language,
  /// for windows built as copies of it.
  pub fn config(self, app: &tauri::AppHandle) -> Option<tauri::utils::config::WindowConfig> {
    let windows = &tauri::Manager::config(app).app.windows;
    let mut config = windows
      .iter()
      .find(|config| config.label == self.as_str())?
      .clone();
    config.title = self.title();
    Some(config)
  }
}
