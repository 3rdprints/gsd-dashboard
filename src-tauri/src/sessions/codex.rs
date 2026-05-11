use serde_json::Value;

use crate::sessions::{
    add_token_count, apply_record_timestamp, json_i64, json_string, parse_timestamp_ms,
    SessionParseAccumulator,
};

/// Extracts session metadata from a single Codex JSONL record.
pub fn parse_codex_record(value: &Value, accumulator: &mut SessionParseAccumulator) {
    let payload = value.get("payload");
    let session_meta = payload.and_then(|payload| payload.get("session_meta"));

    let timestamp = value
        .get("timestamp")
        .and_then(parse_timestamp_ms)
        .or_else(|| {
            payload
                .and_then(|payload| payload.get("timestamp"))
                .and_then(parse_timestamp_ms)
        })
        .or_else(|| {
            session_meta
                .and_then(|metadata| metadata.get("timestamp"))
                .and_then(parse_timestamp_ms)
        });
    if let Some(timestamp_ms) = timestamp {
        apply_record_timestamp(accumulator, timestamp_ms);
    }

    if accumulator.session.cwd.is_none() {
        accumulator.session.cwd = json_string(
            session_meta
                .and_then(|metadata| metadata.get("cwd"))
                .or_else(|| payload.and_then(|payload| payload.get("cwd")))
                .or_else(|| value.get("cwd")),
        );
    }
    if accumulator.session.source_session_id.is_none() {
        accumulator.session.source_session_id = json_string(
            payload
                .and_then(|payload| payload.get("id"))
                .or(value.get("id")),
        );
    }
    if accumulator.session.model.is_none() {
        accumulator.session.model = json_string(
            payload
                .and_then(|payload| payload.get("model"))
                .or_else(|| payload.and_then(|payload| payload.get("provider_model")))
                .or_else(|| session_meta.and_then(|metadata| metadata.get("model")))
                .or_else(|| value.get("model")),
        );
    }

    let record_type = value.get("type").and_then(Value::as_str);
    if !matches!(record_type, Some("session_meta" | "turn_context")) {
        accumulator.session.message_count += 1;
    }

    if let Some(payload) = payload {
        let info = payload.get("info");
        let usage = payload
            .get("usage")
            .or_else(|| payload.get("token_usage"))
            .or_else(|| info.and_then(|info| info.get("last_token_usage")))
            .or_else(|| info.and_then(|info| info.get("total_token_usage")));
        add_usage(usage, accumulator);
    }
}

fn add_usage(usage: Option<&Value>, accumulator: &mut SessionParseAccumulator) {
    let Some(usage) = usage else {
        return;
    };

    add_token_count(
        &mut accumulator.session.tokens_in,
        json_i64(usage.get("input_tokens")).or_else(|| json_i64(usage.get("tokens_in"))),
    );
    add_token_count(
        &mut accumulator.session.tokens_out,
        json_i64(usage.get("output_tokens"))
            .or_else(|| json_i64(usage.get("tokens_out")))
            .or_else(|| json_i64(usage.get("completion_tokens"))),
    );
}
