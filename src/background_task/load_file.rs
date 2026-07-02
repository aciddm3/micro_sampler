use std::path::Path;
use crate::sampler::decoded_audio::DecodedAudio;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub fn load_file(path: &Path) -> Result<DecodedAudio, Box<dyn std::error::Error>> {
    // открываем файл и заворачиваем в MediaSourceStream
    let src = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    // подсказка для пробера: подсунем расширение, чтобы он легче угадал формат
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    // определяем формат
    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;
    // берём дефолтный аудиотрек
    let track = format.default_track().ok_or("no default track found")?;
	let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.ok_or("sample rate not found")? as f32;
    let channels = track.codec_params.channels.ok_or("channels not found")?;
    let channel_count = channels.count();

    // создаём декодер под кодек этого трека
    let mut decoder = symphonia::default::get_codecs().make(
        &track.codec_params,
        &DecoderOptions::default(),
    )?;

    // готовим буферы по каналам
    let mut all_samples: Vec<Vec<f32>> = vec![Vec::new(); channel_count];

    // Предвыделение, если количество фреймов известно заранее (wav, flac обычно знают)
    if let Some(n_frames) = track.codec_params.n_frames {
        for ch in &mut all_samples {
            ch.reserve(n_frames as usize);
        }
    }

    // крутим цикл декодирования
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        };

        // пропускаем пакеты из других треков (обложки альбомов и прочее ненужное)
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(Error::DecodeError(_)) => continue, // битый кадр — пропускаем
            Err(e) => return Err(Box::new(e)),
        };

        // перегоняем декодированные сэмплы в f32
        let spec = *decoded.spec();
        let frames = decoded.frames();

        let mut sample_buf = SampleBuffer::<f32>::new(frames as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        let samples = sample_buf.samples();

        // разбиваем интерлиованный поток (Ch1, Ch2, Ch1, Ch2, ...) по каналам
        for (i, &sample) in samples.iter().enumerate() {
            all_samples[i % channel_count].push(sample);
        }
    }

    Ok(DecodedAudio::new(all_samples, sample_rate))
}