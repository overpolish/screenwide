// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The jobs the helper does on a camera recording: its matte, the objects
//! picked in it, and where the face is.

use screenwide_transcriber::{Job, Reply};

use super::{Told, Transcriber};

impl Transcriber {
  /// Runs `job`, a [`Job::MatteCamera`], which writes each chunk's matte,
  /// telling `progress` how far it has got and `chunk_done` each chunk's
  /// place in the job once its matte is written.
  pub fn matte_camera(
    &mut self,
    job: Job,
    progress: &mut dyn FnMut(f32),
    chunk_done: &mut dyn FnMut(usize),
  ) -> Result<(), String> {
    if !matches!(job, Job::MatteCamera { .. }) {
      return Err("Only a camera can be matted".to_owned());
    }
    let mut told = |told: Told| {
      if let Told::Chunk(index) = told {
        chunk_done(index);
      }
    };
    match self.ask_telling(job, progress, &mut told)? {
      Reply::Matted { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// Runs `job`, a [`Job::FollowObjects`] or a [`Job::ComposeMatte`], which
  /// writes its files, telling `progress` how far it has got and, following
  /// objects, `halfway` once the masks after the click are written.
  pub fn camera_objects(
    &mut self,
    job: Job,
    progress: &mut dyn FnMut(f32),
    halfway: &mut dyn FnMut(),
  ) -> Result<(), String> {
    if !matches!(job, Job::FollowObjects { .. } | Job::ComposeMatte { .. }) {
      return Err("Only a camera's objects are asked for here".to_owned());
    }
    let mut told = |told: Told| {
      if let Told::Halfway = told {
        halfway();
      }
    };
    match self.ask_telling(job, progress, &mut told)? {
      Reply::Tracked { .. } | Reply::Composed { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// Runs `job`, a [`Job::FindFaces`], which writes the camera's face track,
  /// telling `progress` how far it has got.
  pub fn find_faces(&mut self, job: Job, progress: &mut dyn FnMut(f32)) -> Result<(), String> {
    if !matches!(job, Job::FindFaces { .. }) {
      return Err("Only a camera's faces are asked for here".to_owned());
    }
    match self.ask(job, progress)? {
      Reply::FacesFound { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }
}
