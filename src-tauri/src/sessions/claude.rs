use serde_json::Value;

use crate::sessions::{
    add_token_count, apply_record_timestamp, json_i64, json_string, parse_timestamp_ms,
    SessionParseAccumulator,
};

pub fn parse_claude_record(value: &Value, accumulator: &mut SessionParseAccumulator) {
    if let Some(timestamp_ms) = value.get("timestamp").and_then(parse_timestamp_ms) {
        apply_record_timestamp(accumulator, timestamp_ms);
    }

    if accumulator.session.cwd.is_none() {
        accumulator.session.cwd = json_string(value.get("cwd"));
    }
    if accumulator.session.source_session_id.is_none() {
        accumulator.session.source_session_id = json_string(value.get("sessionId"));
    }

    let record_type = value.get("type").and_then(Value::as_str);
    if matches!(record_type, Some("user" | "assistant" | "message")) {
        accumulator.session.message_count += 1;
    }

    let message = value.get("message");
    if accumulator.session.model.is_none() {
        accumulator.session.model = json_string(
            message
                .and_then(|message| message.get("model"))
                .or(value.get("model")),
        );
    }

    add_usage(value.get("usage"), accumulator);
    if let Some(message) = message {
        add_usage(message.get("usage"), accumulator);
    }
}

fn add_usage(usage: Option<&Value>, accumulator: &mut SessionParseAccumulator) {
    let Some(usage) = usage else {
        return;
    };

    add_token_count(
        &mut accumulator.session.tokens_in,
        json_i64(usage.get("input_tokens")),
    );
    add_token_count(
        &mut accumulator.session.tokens_in,
        json_i64(usage.get("cache_creation_input_tokens")),
    );
    add_token_count(
        &mut accumulator.session.tokens_in,
        json_i64(usage.get("cache_read_input_tokens")),
    );
    add_token_count(
        &mut accumulator.session.tokens_out,
        json_i64(usage.get("output_tokens")),
    );
}
