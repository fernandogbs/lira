use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use lira_core::settings::SttModel;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelId {
    LargeV3TurboQ5_0,
    SmallQ5_1,
    VadSilero,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelSpec {
    pub id: ModelId,
    pub name: &'static str,
    pub file: &'static str,
    pub url: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

impl ModelId {
    pub fn spec(self) -> &'static ModelSpec {
        match self {
            Self::LargeV3TurboQ5_0 => &LARGE_V3_TURBO_Q5_0,
            Self::SmallQ5_1 => &SMALL_Q5_1,
            Self::VadSilero => &VAD_SILERO,
        }
    }
}

impl From<SttModel> for ModelId {
    fn from(value: SttModel) -> Self {
        match value {
            SttModel::LargeV3TurboQ5_0 => Self::LargeV3TurboQ5_0,
            SttModel::SmallQ5_1 => Self::SmallQ5_1,
        }
    }
}

pub static LARGE_V3_TURBO_Q5_0: ModelSpec = ModelSpec {
    id: ModelId::LargeV3TurboQ5_0,
    name: "large-v3-turbo-q5_0",
    file: "ggml-large-v3-turbo-q5_0.bin",
    url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin",
    size: 574_041_195,
    sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
};

pub static SMALL_Q5_1: ModelSpec = ModelSpec {
    id: ModelId::SmallQ5_1,
    name: "small-q5_1",
    file: "ggml-small-q5_1.bin",
    url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin",
    size: 190_085_487,
    sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
};

pub static VAD_SILERO: ModelSpec = ModelSpec {
    id: ModelId::VadSilero,
    name: "silero-v5.1.2",
    file: "ggml-silero-v5.1.2.bin",
    url: "https://huggingface.co/ggml-org/whisper-vad/resolve/main/ggml-silero-v5.1.2.bin",
    size: 885_098,
    sha256: "29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf",
};

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("download failed: {0}")]
    Download(String),
    #[error("checksum mismatch for {0}")]
    ChecksumMismatch(String),
    #[error("no data directory")]
    NoDataDir,
}

pub fn models_dir() -> Result<PathBuf, ModelError> {
    if let Ok(dir) = std::env::var("LIRA_MODELS_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let base = dirs::data_dir().ok_or(ModelError::NoDataDir)?;
    Ok(base.join("lira").join("models"))
}

pub fn model_path(spec: &ModelSpec) -> Result<PathBuf, ModelError> {
    Ok(models_dir()?.join(spec.file))
}

pub fn verify_file(path: &Path, spec: &ModelSpec) -> Result<(), ModelError> {
    let meta = fs::metadata(path)?;
    if meta.len() != spec.size {
        return Err(ModelError::ChecksumMismatch(spec.file.to_string()));
    }
    let hash = sha256_file(path)?;
    if hash != spec.sha256 {
        return Err(ModelError::ChecksumMismatch(spec.file.to_string()));
    }
    Ok(())
}

pub fn sha256_file(path: &Path) -> Result<String, ModelError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    while let Ok(n) = file.read(&mut buf) {
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn ensure_model(spec: &ModelSpec) -> Result<PathBuf, ModelError> {
    fs::create_dir_all(models_dir()?)?;
    let dest = model_path(spec)?;
    if dest.exists() {
        let meta = fs::metadata(&dest)?;
        if meta.len() == spec.size {
            return Ok(dest);
        }
        fs::remove_file(&dest)?;
    }

    let part = dest.with_extension("part");
    if part.exists() {
        fs::remove_file(&part)?;
    }

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(10)))
        .build()
        .into();
    let mut response = agent
        .get(spec.url)
        .call()
        .map_err(|e| ModelError::Download(e.to_string()))?;
    if !response.status().is_success() {
        return Err(ModelError::Download(format!(
            "HTTP {} for {}",
            response.status(),
            spec.url
        )));
    }

    let mut reader = response.body_mut().as_reader();
    let mut file = File::create(&part)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| ModelError::Download(e.to_string()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n])?;
    }
    file.sync_all()?;

    let hash = hex::encode(hasher.finalize());
    if hash != spec.sha256 {
        fs::remove_file(&part)?;
        return Err(ModelError::ChecksumMismatch(spec.file.to_string()));
    }

    fs::rename(part, &dest)?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static MODELS_DIR_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn ensure_model_size_fast_path_skips_hash() {
        let _guard = MODELS_DIR_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let spec = ModelSpec {
            id: ModelId::VadSilero,
            name: "fast-path",
            file: "fast-path.bin",
            url: "",
            size: 64,
            sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        };
        fs::write(dir.path().join(spec.file), vec![0u8; 64]).unwrap();
        std::env::set_var("LIRA_MODELS_DIR", dir.path());
        let path = ensure_model(&spec).unwrap();
        assert_eq!(path, dir.path().join(spec.file));
        std::env::remove_var("LIRA_MODELS_DIR");
    }

    #[test]
    fn verify_rejects_wrong_hash() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(VAD_SILERO.file);
        fs::write(&path, b"not the model").unwrap();
        assert!(verify_file(&path, &VAD_SILERO).is_err());
    }

    #[test]
    fn verify_accepts_matching_hash() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tiny.bin");
        let data = b"hello checksum test";
        fs::write(&path, data).unwrap();
        let digest = sha256_file(&path).unwrap();
        let spec = ModelSpec {
            id: ModelId::VadSilero,
            name: "test",
            file: "tiny.bin",
            url: "",
            size: data.len() as u64,
            sha256: "48ceffc2b55ce479c26c0ecc46a7f7d5af280edfd9b554039a7ce02e7670b6ea",
        };
        assert_eq!(digest, spec.sha256);
        verify_file(&path, &spec).unwrap();
    }
}
