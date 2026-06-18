use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    traits::RequestOptionsBuilder,
    types::{
        chat::{
            ChatCompletionRequestMessage, ChatCompletionRequestUserMessage,
            ChatCompletionRequestUserMessageContent, Choice, CompletionFinishReason,
            CreateChatCompletionRequest, CreateChatCompletionResponse, FinishReason, Prompt,
        },
        completions::{CreateCompletionRequest, CreateCompletionResponse},
    },
};
use axum::{
    Json, Router,
    extract::State,
    http::{
        HeaderMap, StatusCode,
        header::{CONTENT_ENCODING, CONTENT_LENGTH, HOST, ORIGIN},
    },
    routing::post,
};
use eyre::Result;
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let listen_addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| String::from("0.0.0.0:3000"));

    let app = Router::new()
        .route("/v1/completions", post(completions))
        .with_state(Client::new());

    let listener = TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn completions(
    State(client): State<Client<OpenAIConfig>>,
    mut headers: HeaderMap,
    Json(request): Json<CreateCompletionRequest>,
) -> Result<Json<CreateCompletionResponse>, StatusCode> {
    // Remove headers that should not be forwarded upstream.
    headers.remove(CONTENT_ENCODING);
    headers.remove(CONTENT_LENGTH);
    headers.remove(HOST);
    headers.remove(ORIGIN);

    // Forward the request to the provider.
    match client
        .chat()
        .headers(headers)
        .create(convert_request(request))
        .await
    {
        Ok(response) => Ok(Json(convert_response(response))),
        Err(OpenAIError::ApiError(response)) => Err(response.status_code),
        Err(error) => {
            tracing::error!(?error, "error sending upstream request");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[allow(deprecated)]
fn convert_request(request: CreateCompletionRequest) -> CreateChatCompletionRequest {
    CreateChatCompletionRequest {
        model: request.model,
        messages: match request.prompt {
            Prompt::String(prompt) => {
                vec![ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        name: None,
                        content: ChatCompletionRequestUserMessageContent::Text(prompt),
                    },
                )]
            }
            Prompt::StringArray(prompts) => prompts
                .into_iter()
                .map(|prompt| {
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        name: None,
                        content: ChatCompletionRequestUserMessageContent::Text(prompt),
                    })
                })
                .collect(),
            _ => todo!(),
        },
        max_tokens: request.max_tokens,
        n: request.n,
        seed: request.seed,
        stop: request.stop,
        temperature: request.temperature,
        ..CreateChatCompletionRequest::default()
    }
}

#[allow(deprecated)]
fn convert_response(response: CreateChatCompletionResponse) -> CreateCompletionResponse {
    CreateCompletionResponse {
        id: response.id,
        choices: response
            .choices
            .into_iter()
            .map(|chat_choice| Choice {
                text: chat_choice.message.content.unwrap_or_default(),
                index: chat_choice.index,
                finish_reason: match chat_choice.finish_reason {
                    Some(FinishReason::Stop) => Some(CompletionFinishReason::Stop),
                    Some(FinishReason::Length) => Some(CompletionFinishReason::Length),
                    Some(FinishReason::ContentFilter) => {
                        Some(CompletionFinishReason::ContentFilter)
                    }
                    _ => None,
                },
                logprobs: None, // not implemented
            })
            .collect(),
        created: response.created,
        model: response.model,
        system_fingerprint: response.system_fingerprint,
        object: String::from("text_completion"),
        usage: response.usage,
    }
}
