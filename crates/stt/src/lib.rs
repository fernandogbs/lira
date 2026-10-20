use async_trait::async_trait;

#[async_trait]
pub trait Transcriber: Send + Sync {
    async fn transcribe(&self, _pcm: &[u8]) -> Result<String, String> {
        Err("transcriber not implemented".into())
    }
}
