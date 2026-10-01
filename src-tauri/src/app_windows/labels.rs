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
  /// The shared confirmation sheet, a child of whichever window asked.
  ConfirmSheet,
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
    Self::ConfirmSheet,
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
      Self::ConfirmSheet => "confirm-sheet",
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
      Self::Settings => "settings",
      Self::RegionSelector => "region-selector",
      Self::RecordingSourceSelector => "recording-source-selector",
      Self::StandaloneListbox => "standalone-listbox",
      Self::TextRecognition => "text-recognition",
      Self::Update => "update",
    }
  }
}
