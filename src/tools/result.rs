use rmcp::model::ErrorData;
use serde::Serialize;

use crate::error::HmsError;

pub fn ok_json<T: Serialize>(value: &T) -> Result<String, ErrorData> {
    serde_json::to_string_pretty(value).map_err(|err| failure(HmsError::message(err.to_string())))
}

pub fn failure(err: HmsError) -> ErrorData {
    ErrorData::internal_error(err.to_string(), None)
}

pub fn pack<T: Serialize>(result: Result<T, HmsError>) -> Result<String, ErrorData> {
    result.map_err(failure).and_then(|value| ok_json(&value))
}
