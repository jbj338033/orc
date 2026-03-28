use std::pin::Pin;

use futures::Stream;
use tokio_util::sync::CancellationToken;

use super::registry::ProviderRegistry;
use super::types::{CompletionRequest, ModelHandle, ProviderError, StreamPart};

pub struct AgentModelConfig {
    pub primary: ModelHandle,
    pub fallbacks: Vec<ModelHandle>,
}

pub struct Router {
    registry: ProviderRegistry,
}

impl Router {
    pub fn new(registry: ProviderRegistry) -> Self {
        Self { registry }
    }

    pub async fn stream(
        &self,
        config: &AgentModelConfig,
        mut request: CompletionRequest,
        cancel: CancellationToken,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamPart, ProviderError>> + Send>>, ProviderError>
    {
        let chain = std::iter::once(&config.primary).chain(config.fallbacks.iter());

        let mut last_error = None;

        for handle in chain {
            if cancel.is_cancelled() {
                return Err(ProviderError {
                    code: super::types::ErrorCode::Unknown,
                    message: "cancelled".into(),
                    retriable: false,
                });
            }

            let provider = match self.registry.get(&handle.provider) {
                Ok(p) => p,
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            };

            request.model = handle.model.clone();

            match provider.stream(request.clone(), cancel.clone()).await {
                Ok(stream) => return Ok(stream),
                Err(e) if e.retriable => {
                    last_error = Some(e);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or_else(|| ProviderError {
            code: super::types::ErrorCode::Unknown,
            message: "all providers failed".into(),
            retriable: false,
        }))
    }

    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut ProviderRegistry {
        &mut self.registry
    }
}
