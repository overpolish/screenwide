// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reads requests from standard input until it closes, answering each on
//! standard output. Engine logs are dropped: standard error is kept for this
//! program's own failures, which the app shows.

mod clean;
mod engine;
mod faces;
mod matte;
mod restore;
mod separate;
mod session;

use std::io::{BufRead, Write};

use screenwide_transcriber::{Job, Reply, Request};

fn main() {
  whisper_rs::install_logging_hooks();
  let mut engine = engine::Engine::default();
  let stdin = std::io::stdin();
  let mut stdout = std::io::stdout().lock();
  for line in stdin.lock().lines() {
    let Ok(line) = line else { break };
    if line.trim().is_empty() {
      continue;
    }
    let reply = match serde_json::from_str::<Request>(&line) {
      Ok(Request { id, job }) => {
        // Progress and, for following objects, word that half is done, are
        // both sent while the job runs.
        let out = std::cell::RefCell::new(&mut stdout);
        let mut progress = |fraction: f32| {
          send(&mut out.borrow_mut(), &Reply::Progress { id, fraction });
        };
        let mut halfway = || {
          send(&mut out.borrow_mut(), &Reply::Halfway { id });
        };
        let result = match &job {
          Job::Transcribe {
            model,
            model_id,
            audio,
            language,
            stretches,
          } => engine
            .transcribe(
              model,
              model_id,
              audio,
              language.as_deref(),
              stretches,
              &mut progress,
            )
            .map(|transcript| Reply::Done { id, transcript }),
          Job::DetectSpeech { model, audio } => engine
            .detect_speech(model, audio, &mut progress)
            .map(|speech| Reply::Speech { id, speech }),
          Job::CleanSpeech { audio, output } => clean::clean_speech(audio, output, &mut progress)
            .map(|gain| Reply::Cleaned { id, gain }),
          Job::RestoreSpeech {
            features,
            decoder,
            audio,
            output,
            stretches,
          } => {
            restore::restore_speech((features, decoder), audio, stretches, output, &mut progress)
              .map(|()| Reply::Restored { id })
          }
          Job::SeparateVocals {
            model,
            audio,
            output,
          } => separate::separate_vocals(model, audio, output, &mut progress)
            .map(|()| Reply::Separated { id }),
          Job::MatteCamera {
            model,
            ffmpeg,
            camera,
            width,
            height,
            frames,
            frames_per_second,
            chunks,
          } => matte::matte_camera(
            (model, ffmpeg),
            camera,
            (*width, *height),
            (*frames, *frames_per_second),
            chunks,
            &mut progress,
            &mut |index| {
              send(&mut out.borrow_mut(), &Reply::ChunkMatted { id, index });
            },
          )
          .map(|()| Reply::Matted { id }),
          Job::FollowObjects {
            model,
            ffmpeg,
            camera,
            timings,
            frame,
            objects,
          } => {
            let clicked = objects
              .iter()
              .map(|object| matte::Clicked {
                points: &object.points,
                after: &object.after,
                before: &object.before,
              })
              .collect::<Vec<_>>();
            matte::follow_objects(
              &matte::Follow {
                model,
                ffmpeg,
                camera,
                timings,
                frame: *frame,
                objects: &clicked,
              },
              &mut progress,
              &mut halfway,
            )
            .map(|()| Reply::Tracked { id })
          }
          Job::ComposeMatte {
            ffmpeg,
            camera,
            matte,
            width,
            height,
            frames,
            frames_per_second,
            picks,
            output,
          } => matte::compose_matte(
            (ffmpeg, camera),
            matte,
            ((*width, *height), *frames, *frames_per_second),
            picks,
            output,
            &mut progress,
          )
          .map(|()| Reply::Composed { id }),
          Job::FindFaces {
            model,
            ffmpeg,
            camera,
            width,
            height,
            duration_ms,
            output,
          } => faces::find_faces(
            (model, ffmpeg),
            camera,
            (*width, *height),
            *duration_ms,
            output,
            &mut progress,
          )
          .map(|()| Reply::FacesFound { id }),
        };
        result.unwrap_or_else(|message| Reply::Failed { id, message })
      }
      // Without an id there is no request to answer; the app sees the
      // helper go and gives up on whatever it was waiting for.
      Err(error) => {
        eprintln!("Unreadable request: {error}");
        std::process::exit(2);
      }
    };
    if !send(&mut stdout, &reply) {
      break;
    }
  }
}

/// Writes `reply` as one line. False once the app has stopped listening.
fn send(stdout: &mut std::io::StdoutLock<'_>, reply: &Reply) -> bool {
  let Ok(json) = serde_json::to_string(reply) else {
    std::process::exit(3);
  };
  writeln!(stdout, "{json}")
    .and_then(|()| stdout.flush())
    .is_ok()
}
