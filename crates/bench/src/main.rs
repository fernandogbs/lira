mod wer;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::{Parser, ValueEnum};
use hound::{SampleFormat, WavReader};
use lira_audio::resample_to_16k_mono;
use lira_stt::{ensure_model, stt_n_threads, WhisperEngine};
use serde::Serialize;

const SAMPLE_RATE: u32 = 16_000;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BenchModel {
    Turbo,
    Small,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BenchLang {
    Pt,
    En,
}

impl BenchLang {
    fn whisper_code(self) -> &'static str {
        match self {
            Self::Pt => "pt",
            Self::En => "en",
        }
    }
}

#[derive(Parser)]
#[command(name = "lira-bench", about = "Lira STT benchmark")]
struct Args {
    #[arg(long)]
    manifest: PathBuf,
    #[arg(long, value_enum)]
    model: BenchModel,
    #[arg(long, value_enum)]
    lang: BenchLang,
    #[arg(long, default_value_t = false)]
    gpu: bool,
    #[arg(long)]
    label: Option<String>,
    #[arg(long, default_value_t = 1)]
    warmup: u32,
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long)]
    threads: Option<usize>,
    #[arg(long, default_value_t = false)]
    audio_ctx_auto: bool,
}

#[derive(Serialize)]
struct BenchResult {
    label: String,
    model: String,
    lang: String,
    gpu: bool,
    threads: usize,
    audio_ctx_auto: bool,
    files: u64,
    audio_secs: f64,
    total_secs: f64,
    secs_per_10s_audio: f64,
    wer: f64,
    peak_rss_mib: f64,
}

fn peak_rss_mib() -> f64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if let Some(kib) = line.strip_prefix("VmHWM:") {
            let kib = kib.trim().strip_suffix(" kB").unwrap_or(kib.trim());
            if let Ok(v) = kib.parse::<f64>() {
                return v / 1024.0;
            }
        }
    }
    0.0
}

fn load_wav_pcm_16k(path: &Path) -> Result<(Vec<f32>, f64), String> {
    let mut reader = WavReader::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let spec = reader.spec();
    if spec.channels != 1 {
        return Err(format!(
            "{}: expected mono, got {} channels",
            path.display(),
            spec.channels
        ));
    }

    let samples: Vec<f32> = match spec.sample_format {
        SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
        SampleFormat::Int => {
            let scale = 1i32 << (spec.bits_per_sample - 1);
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / scale as f32))
                .map(|s| s.map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    let rate = spec.sample_rate;
    let pcm = if rate == SAMPLE_RATE {
        samples
    } else {
        resample_to_16k_mono(&samples, rate)
    };

    let audio_secs = pcm.len() as f64 / SAMPLE_RATE as f64;
    Ok((pcm, audio_secs))
}

fn model_spec(model: BenchModel) -> &'static lira_stt::ModelSpec {
    match model {
        BenchModel::Turbo => &lira_stt::LARGE_V3_TURBO_Q5_0,
        BenchModel::Small => &lira_stt::SMALL_Q5_1,
    }
}

fn read_manifest(path: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("{}: line {i}: {e}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let Some((wav, ref_text)) = line.split_once('\t') else {
            return Err(format!(
                "{}: line {i}: expected wav<TAB>ref",
                path.display()
            ));
        };
        rows.push((PathBuf::from(wav), ref_text.to_string()));
    }
    Ok(rows)
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    if let Some(n) = args.threads {
        // SAFETY: single-threaded CLI; no other threads read this env var yet.
        unsafe { std::env::set_var("LIRA_STT_THREADS", n.to_string()) };
    }
    let spec = model_spec(args.model);
    ensure_model(spec).map_err(|e| e.to_string())?;
    let engine = WhisperEngine::load_model(spec, args.gpu).map_err(|e| e.to_string())?;
    let threads = stt_n_threads();

    let mut rows = read_manifest(&args.manifest)?;
    if let Some(limit) = args.limit {
        rows.truncate(limit);
    }
    if rows.is_empty() {
        return Err("manifest is empty".into());
    }

    let lang = args.lang.whisper_code();
    let model_name = match args.model {
        BenchModel::Turbo => "turbo",
        BenchModel::Small => "small",
    };
    let label = args
        .label
        .clone()
        .unwrap_or_else(|| format!("{}-{}", model_name, if args.gpu { "gpu" } else { "cpu" }));

    for _ in 0..args.warmup {
        let (pcm, _) = load_wav_pcm_16k(&rows[0].0)?;
        engine
            .transcribe(&pcm, lang, None, args.audio_ctx_auto)
            .map_err(|e| e.to_string())?;
    }

    let mut refs = Vec::new();
    let mut hyps = Vec::new();
    let mut total_wall = 0.0;
    let mut audio_secs = 0.0;

    for (wav, reference) in &rows {
        let (pcm, dur) = load_wav_pcm_16k(wav)?;
        audio_secs += dur;
        let t0 = Instant::now();
        let hyp = engine
            .transcribe(&pcm, lang, None, args.audio_ctx_auto)
            .map_err(|e| e.to_string())?;
        let elapsed = t0.elapsed().as_secs_f64();
        total_wall += elapsed;
        eprintln!("{}\t{:.3}s\t{}", wav.display(), elapsed, hyp);
        refs.push(reference.clone());
        hyps.push(hyp);
    }

    let wer_score = wer::wer(&refs, &hyps);
    let secs_per_10s = if audio_secs > 0.0 {
        total_wall / audio_secs * 10.0
    } else {
        0.0
    };

    let result = BenchResult {
        label,
        model: match args.model {
            BenchModel::Turbo => "turbo".into(),
            BenchModel::Small => "small".into(),
        },
        lang: match args.lang {
            BenchLang::Pt => "pt".into(),
            BenchLang::En => "en".into(),
        },
        gpu: args.gpu,
        threads,
        audio_ctx_auto: args.audio_ctx_auto,
        files: rows.len() as u64,
        audio_secs,
        total_secs: total_wall,
        secs_per_10s_audio: secs_per_10s,
        wer: wer_score,
        peak_rss_mib: peak_rss_mib(),
    };

    println!(
        "{}",
        serde_json::to_string(&result).map_err(|e| e.to_string())?
    );
    Ok(())
}
