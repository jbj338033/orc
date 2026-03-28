use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use tokio_util::sync::CancellationToken;

use super::types::{CompletionRequest, ProviderError, StreamPart};

#[async_trait]
pub trait CompletionProvider: Send + Sync {
    async fn stream(
        &self,
        request: CompletionRequest,
        cancel: CancellationToken,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamPart, ProviderError>> + Send>>, ProviderError>;

    fn provider_name(&self) -> &str;
}
