use base64::Engine;

pub(crate) fn encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub(crate) fn decode(input: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .ok()
        .or_else(|| base64::engine::general_purpose::URL_SAFE.decode(input).ok())
}
