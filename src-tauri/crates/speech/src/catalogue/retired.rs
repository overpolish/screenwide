// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What earlier versions kept in the models folder and this one no longer
//! uses, removed so it does not take room for nothing.

/// Files models no longer in the catalogue were kept in: HT Demucs, which
/// Smart ducking ran before, whose weights are licensed for research only,
/// and Sidon, which Restore voice ran before and which wobbled the pitch of
/// a voice it rebuilt.
pub const RETIRED_FILES: &[&str] = &["htdemucs.onnx", "sidon-encoder.onnx", "sidon-vocoder.onnx"];

/// How the folders of compiles kept under names no longer used start:
/// object following's CoreML compiles, before each compile's folder was
/// named after its model's first file (`compiled_model_prefix`).
pub const RETIRED_FOLDER_PREFIXES: &[&str] = &["coreml-cache-"];
