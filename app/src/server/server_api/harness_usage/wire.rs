use serde::Deserialize;

use super::HarnessUsagePublicationStatus;

#[derive(Deserialize)]
pub(super) struct StartupCapability {
    pub execution_id: i64,
}

#[derive(Deserialize)]
pub(super) struct PublicationAcknowledgment {
    pub status: HarnessUsagePublicationStatus,
    pub execution_id: i64,
    pub capture_sequence: i64,
}

#[derive(Deserialize)]
pub(super) struct Problem {
    #[serde(rename = "type", default)]
    pub problem_type: ProblemType,
    pub retryable: Option<bool>,
}

#[derive(Clone, Copy, Default, Deserialize)]
pub(super) enum ProblemType {
    #[serde(rename = "https://github.com/SifG22/brasa")]
    Disabled,
    #[serde(rename = "https://github.com/SifG22/brasa")]
    Unsupported,
    #[default]
    #[serde(other)]
    Unknown,
}
