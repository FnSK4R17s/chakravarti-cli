use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use futures_util::stream::{Stream, StreamExt};
use std::convert::Infallible;
use crate::state::AppState;
use crate::hub::OrchestrationEvent;

pub async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.hub.subscribe();
    
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .map(|msg| {
            match msg {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    Event::default().data(json)
                }
                Err(_) => {
                    // Lagged or closed, we can send a comment or just skip
                    Event::default().comment("lagged")
                }
            }
        })
        .map(Ok);

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
