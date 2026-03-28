use std::collections::HashMap;
use std::sync::Arc;

use futures::StreamExt;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::runtime::{
    AgentEngine, AgentEvent, ContentBlock, EngineRequest, Message, MessageRole, ToolResult,
};
use crate::tool::ToolRegistry;

use super::hook::{Hook, HookDecision};

pub struct AgentLoop {
    engine: Arc<dyn AgentEngine>,
    tools: Arc<ToolRegistry>,
    hooks: Vec<Arc<dyn Hook>>,
    max_iterations: u32,
}

impl AgentLoop {
    pub fn new(
        engine: Arc<dyn AgentEngine>,
        tools: Arc<ToolRegistry>,
        hooks: Vec<Arc<dyn Hook>>,
        max_iterations: u32,
    ) -> Self {
        Self {
            engine,
            tools,
            hooks,
            max_iterations,
        }
    }

    pub async fn run(
        &self,
        mut messages: Vec<Message>,
        system_prompt: Option<&str>,
        max_tokens: u32,
        temperature: f32,
        cancel: CancellationToken,
        event_handler: &dyn Fn(AgentEvent),
    ) -> Vec<Message> {
        let tool_defs = self.tools.definitions();

        for _iteration in 0..self.max_iterations {
            if cancel.is_cancelled() {
                break;
            }

            let request = EngineRequest {
                messages: &messages,
                tools: &tool_defs,
                system_prompt,
                max_tokens,
                temperature,
                cancel: cancel.clone(),
                extensions: HashMap::new(),
            };

            let stream = match self.engine.send(request).await {
                Ok(s) => s,
                Err(e) => {
                    event_handler(AgentEvent::Error(e.to_string()));
                    break;
                }
            };

            let mut stream = std::pin::pin!(stream);
            let mut text_parts = String::new();
            let mut tool_calls: Vec<PendingToolCall> = Vec::new();
            let mut current_tool_id = String::new();
            let mut current_tool_name = String::new();
            let mut current_tool_args = String::new();

            while let Some(event) = stream.next().await {
                match &event {
                    AgentEvent::TextDelta(delta) => {
                        text_parts.push_str(delta);
                        event_handler(event);
                    }
                    AgentEvent::ToolCall { id, name, input } => {
                        tool_calls.push(PendingToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        });
                        event_handler(event);
                    }
                    AgentEvent::Done | AgentEvent::Error(_) => {
                        // flush any accumulated tool call delta
                        if !current_tool_id.is_empty() {
                            if let Ok(args) = serde_json::from_str(&current_tool_args) {
                                tool_calls.push(PendingToolCall {
                                    id: current_tool_id.clone(),
                                    name: current_tool_name.clone(),
                                    input: args,
                                });
                            }
                            current_tool_id.clear();
                            current_tool_name.clear();
                            current_tool_args.clear();
                        }
                        event_handler(event);
                    }
                    _ => {
                        event_handler(event);
                    }
                }
            }

            // build assistant message
            let mut content = Vec::new();
            if !text_parts.is_empty() {
                content.push(ContentBlock::Text { text: text_parts });
            }
            for tc in &tool_calls {
                content.push(ContentBlock::ToolUse {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    input: tc.input.clone(),
                });
            }

            if !content.is_empty() {
                messages.push(Message {
                    role: MessageRole::Assistant,
                    content,
                });
            }

            // no tool calls = done
            if tool_calls.is_empty() {
                break;
            }

            // execute tools
            let results = self.execute_tools(&tool_calls, &cancel).await;

            // build user message with tool results
            let result_content: Vec<ContentBlock> = results
                .into_iter()
                .map(|(id, result)| {
                    event_handler(AgentEvent::ToolResult {
                        id: id.clone(),
                        output: result.output.clone(),
                        is_error: result.is_error,
                    });
                    ContentBlock::ToolResult {
                        id,
                        output: result.output,
                        is_error: result.is_error,
                    }
                })
                .collect();

            messages.push(Message {
                role: MessageRole::User,
                content: result_content,
            });
        }

        // on_stop hooks
        for hook in &self.hooks {
            hook.on_stop().await;
        }

        messages
    }

    async fn execute_tools(
        &self,
        tool_calls: &[PendingToolCall],
        cancel: &CancellationToken,
    ) -> Vec<(String, ToolResult)> {
        // classify: read-only tools run in parallel, write tools run sequentially
        let read_only = ["read", "grep", "glob", "web_fetch"];

        let mut parallel_calls: Vec<&PendingToolCall> = Vec::new();
        let mut sequential_calls: Vec<&PendingToolCall> = Vec::new();

        for tc in tool_calls {
            if read_only.contains(&tc.name.as_str()) {
                parallel_calls.push(tc);
            } else {
                sequential_calls.push(tc);
            }
        }

        let mut results: Vec<(String, ToolResult)> = Vec::new();

        // parallel execution for read-only tools
        if !parallel_calls.is_empty() {
            let mut join_set = JoinSet::new();

            for tc in &parallel_calls {
                let tool = self.tools.get(&tc.name).cloned();
                let input = tc.input.clone();
                let id = tc.id.clone();
                let name = tc.name.clone();
                let cancel = cancel.clone();
                let hooks = self.hooks.clone();
                let tool_def_opt = tool.as_ref().map(|t| t.definition());

                join_set.spawn(async move {
                    // pre_tool_use hooks
                    if let Some(ref def) = tool_def_opt {
                        for hook in &hooks {
                            if let HookDecision::Deny(reason) =
                                hook.pre_tool_use(def, &input).await
                            {
                                return (
                                    id,
                                    ToolResult {
                                        id: String::new(),
                                        output: format!("denied by hook: {reason}"),
                                        is_error: true,
                                    },
                                );
                            }
                        }
                    }

                    let result = match tool {
                        Some(t) => t.execute(input.clone(), cancel).await,
                        None => ToolRegistry::not_found(&name),
                    };

                    // post_tool_use hooks
                    if let Some(ref def) = tool_def_opt {
                        for hook in &hooks {
                            hook.post_tool_use(def, &input, &result).await;
                        }
                    }

                    (id, result)
                });
            }

            while let Some(Ok(r)) = join_set.join_next().await {
                results.push(r);
            }
        }

        // sequential execution for write tools
        for tc in &sequential_calls {
            let tool_def = self.tools.get(&tc.name).map(|t| t.definition());

            // pre_tool_use hooks
            if let Some(ref def) = tool_def {
                let mut denied = false;
                for hook in &self.hooks {
                    if let HookDecision::Deny(reason) = hook.pre_tool_use(def, &tc.input).await {
                        results.push((
                            tc.id.clone(),
                            ToolResult {
                                id: String::new(),
                                output: format!("denied by hook: {reason}"),
                                is_error: true,
                            },
                        ));
                        denied = true;
                        break;
                    }
                }
                if denied {
                    continue;
                }
            }

            let result = match self.tools.get(&tc.name) {
                Some(t) => t.execute(tc.input.clone(), cancel.clone()).await,
                None => ToolRegistry::not_found(&tc.name),
            };

            // post_tool_use hooks
            if let Some(ref def) = tool_def {
                for hook in &self.hooks {
                    hook.post_tool_use(def, &tc.input, &result).await;
                }
            }

            results.push((tc.id.clone(), result));
        }

        results
    }
}

struct PendingToolCall {
    id: String,
    name: String,
    input: serde_json::Value,
}
