use std::path::Path;

use nih_plug::{nih_error, nih_trace, plugin::TaskExecutor};

use crate::sampler::{Sampler, decoded_audio::DecodedAudio};

mod load_file;
use self::load_file::load_file;

pub enum BackgroundTask {
    LoadSampleWithDialog,
    LoadSampleWithoutDialog,
}

pub trait SamplerTaskExecutor {
    fn task_executor(&mut self) -> TaskExecutor<Sampler>;
}

impl SamplerTaskExecutor for Sampler {
    fn task_executor(&mut self) -> TaskExecutor<Sampler> {
        let params = self.params.clone();
        let decoded_audio_arc = self.decoded_audio.clone();
        Box::new(move |task| match task {
            BackgroundTask::LoadSampleWithoutDialog => {
                if let Some(path) = &*params.file_path.read()
                    && !path.is_empty()
                {
                    match load_file(Path::new(&path)) {
                        Ok(val) => {
                            *decoded_audio_arc.write() = val;
                        }
                        Err(err) => {
                            nih_error!("couldn\'t load file: {}, {err}", path);
                            *decoded_audio_arc.write() =
                                DecodedAudio::new(vec![vec![-1.0, 1.0]], 440.0);
                            *params.file_path.write() = None;
                        }
                    }
                } else {
                    nih_trace!("no file produced");
                }
            }
            BackgroundTask::LoadSampleWithDialog => {
                let fd = rfd::FileDialog::new();
                
                let Some(path_buff) = fd.pick_file() else {
                    nih_error!("Couldn\'t pick a file");
                    return;
                };
                match load_file(&path_buff) {
                    Ok(val) => {
                        *decoded_audio_arc.write() = val;
                        *params.file_path.write() =
                            Some(path_buff.to_str().unwrap_or("").to_string());
                    }
                    Err(err) => {
                        nih_error!(
                            "Error loading file {} : {err}",
                            path_buff.to_str().unwrap_or("")
                        )
                    }
                }
            }
        })
    }
}
