use brasa_core::telemetry::{EnablementState, TelemetryEvent, TelemetryEventDesc};
use serde_json::{Value, json};
use strum_macros::{EnumDiscriminants, EnumIter};

use crate::features::FeatureFlag;

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
pub(super) enum CliTelemetryEvent {
    /// Executing `brasa agent run`
    AgentRun {
        gui: bool,
        requested_mcp_servers: usize,
        has_environment: bool,
        /// Optional task ID when running against an ambient agent task.
        task_id: Option<String>,
        /// Which execution harness was selected (e.g. "oz", "claude").
        harness: String,
    },
    /// Executing `brasa agent run-ambient`
    AgentRunAmbient,
    /// Executing `brasa agent run-cloud-standby`
    AgentRunCloudStandby,
    /// Executing `brasa agent profile list`
    AgentProfileList,
    /// Executing `brasa agent list`
    AgentList,
    /// Executing `brasa agent get`
    AgentGet,
    /// Executing `brasa agent create`
    AgentCreate,
    /// Executing `brasa agent update`
    AgentUpdate,
    /// Executing `brasa agent delete`
    AgentDelete,
    /// Executing `brasa agent skills`
    AgentSkills,
    /// Executing `brasa environment list`
    EnvironmentList,
    /// Executing `brasa environment create`
    EnvironmentCreate,
    /// Executing `brasa environment delete`
    EnvironmentDelete,
    /// Executing `brasa environment update`
    EnvironmentUpdate,
    /// Executing `brasa environment get`
    EnvironmentGet,
    /// Executing `brasa environment image list`
    EnvironmentImageList,
    EnvironmentCheckout,
    /// Executing `brasa mcp list`
    MCPList,
    /// Executing `brasa model list`
    ModelList,
    /// Executing `brasa memory-store list`
    MemoryStoreList,
    /// Executing `brasa memory list`
    MemoryStoreListMemories,
    /// Executing `brasa memory create`
    MemoryStoreCreateMemory,
    /// Executing `brasa memory update`
    MemoryStoreUpdateMemory,
    /// Executing `brasa memory delete`
    MemoryStoreDeleteMemory,
    /// Executing `brasa memory-store get`
    MemoryStoreGetStore,
    /// Executing `brasa memory-store update`
    MemoryStoreUpdateStore,
    /// Executing `brasa memory-store list-store-agents`
    MemoryStoreListStoreAgents,
    /// Executing `brasa memory versions`
    MemoryStoreListVersions,
    /// Executing `brasa task list`
    TaskList,
    /// Executing `brasa task get`
    TaskGet,
    /// Executing `brasa run conversation get`
    ConversationGet,
    /// Executing `brasa run get <id> --conversation`
    RunConversationGet,
    /// Executing `brasa run message watch`
    RunMessageWatch {
        harness: &'static str,
    },
    /// Executing `brasa run message send`
    RunMessageSend {
        harness: &'static str,
    },
    /// Executing `brasa run message list`
    RunMessageList {
        harness: &'static str,
    },
    /// Executing `brasa run message read`
    RunMessageRead {
        harness: &'static str,
    },
    /// Executing `brasa run message mark-delivered`
    RunMessageMarkDelivered {
        harness: &'static str,
    },
    /// Executing `brasa login`
    Login,
    /// Executing `brasa logout`
    Logout,
    /// Executing `brasa whoami`
    Whoami,
    /// Executing `brasa provider setup`
    ProviderSetup,
    /// Executing `brasa provider list`
    ProviderList,
    /// Executing `brasa integration create`
    IntegrationCreate,
    /// Executing `brasa integration update`
    IntegrationUpdate,
    /// Executing `brasa integration list`
    IntegrationList,
    /// Executing `brasa artifact upload`
    ArtifactUpload,
    /// Executing `brasa artifact get`
    ArtifactGet,
    /// Executing `brasa artifact download`
    ArtifactDownload,
    /// Executing `brasa api-key list`
    ApiKeyList,
    /// Executing `brasa api-key create`
    ApiKeyCreate,
    /// Executing `brasa api-key expire`
    ApiKeyExpire,
    /// Executing `brasa schedule create`
    ScheduleCreate,
    /// Executing `brasa schedule list`
    ScheduleList,
    /// Executing `brasa schedule get`
    ScheduleGet,
    /// Executing `brasa schedule pause`
    SchedulePause,
    /// Executing `brasa schedule unpause`
    ScheduleUnpause,
    /// Executing `brasa schedule update`
    ScheduleUpdate,
    /// Executing `brasa schedule delete`
    ScheduleDelete,
    /// Executing `brasa secret create`
    SecretCreate,
    /// Executing `brasa secret delete`
    SecretDelete,
    /// Executing `brasa secret update`
    SecretUpdate,
    /// Executing `brasa secret list`
    SecretList,
    /// Executing `brasa federate issue-token`
    FederateIssueToken,
    /// Executing `brasa federate issue-gcp-token`
    FederateIssueGcpToken,
    /// Executing `brasa harness-support ping`
    HarnessSupportPing,
    /// Executing `brasa harness-support report-artifact`
    HarnessSupportReportArtifact {
        artifact_type: &'static str,
    },
    /// Executing `brasa harness-support notify-user`
    HarnessSupportNotifyUser,
    /// Executing `brasa harness-support finish-task`
    HarnessSupportFinishTask {
        success: bool,
    },
    /// Executing `brasa harness-support report-shutdown`
    HarnessSupportReportShutdown,
    /// Executing `brasa runner list`
    RunnerList,
    /// Executing `brasa runner create`
    RunnerCreate,
    /// Executing `brasa runner update`
    RunnerUpdate,
    /// Executing `brasa runner delete`
    RunnerDelete,
}

impl TelemetryEvent for CliTelemetryEvent {
    fn name(&self) -> &'static str {
        CliTelemetryEventDiscriminants::from(self).name()
    }

    fn payload(&self) -> Option<Value> {
        match self {
            CliTelemetryEvent::AgentRun {
                gui,
                requested_mcp_servers,
                has_environment,
                task_id,
                harness,
            } => Some(json!({
                "gui": gui,
                "requested_mcp_servers": requested_mcp_servers,
                "has_environment": has_environment,
                "task_id": task_id,
                "harness": harness,
            })),
            CliTelemetryEvent::AgentRunAmbient => None,
            CliTelemetryEvent::AgentRunCloudStandby => None,
            CliTelemetryEvent::AgentProfileList => None,
            CliTelemetryEvent::AgentList => None,
            CliTelemetryEvent::AgentGet => None,
            CliTelemetryEvent::AgentCreate => None,
            CliTelemetryEvent::AgentUpdate => None,
            CliTelemetryEvent::AgentDelete => None,
            CliTelemetryEvent::AgentSkills => None,
            CliTelemetryEvent::EnvironmentList => None,
            CliTelemetryEvent::EnvironmentCreate => None,
            CliTelemetryEvent::EnvironmentDelete => None,
            CliTelemetryEvent::EnvironmentUpdate => None,
            CliTelemetryEvent::EnvironmentGet => None,
            CliTelemetryEvent::EnvironmentImageList => None,
            CliTelemetryEvent::EnvironmentCheckout => None,
            CliTelemetryEvent::MCPList => None,
            CliTelemetryEvent::ModelList => None,
            CliTelemetryEvent::MemoryStoreList => None,
            CliTelemetryEvent::MemoryStoreListMemories => None,
            CliTelemetryEvent::MemoryStoreCreateMemory => None,
            CliTelemetryEvent::MemoryStoreUpdateMemory => None,
            CliTelemetryEvent::MemoryStoreDeleteMemory => None,
            CliTelemetryEvent::MemoryStoreGetStore => None,
            CliTelemetryEvent::MemoryStoreUpdateStore => None,
            CliTelemetryEvent::MemoryStoreListStoreAgents => None,
            CliTelemetryEvent::MemoryStoreListVersions => None,
            CliTelemetryEvent::TaskList => None,
            CliTelemetryEvent::TaskGet => None,
            CliTelemetryEvent::ConversationGet => None,
            CliTelemetryEvent::RunConversationGet => None,
            CliTelemetryEvent::RunMessageWatch { harness } => Some(json!({ "harness": harness })),
            CliTelemetryEvent::RunMessageSend { harness } => Some(json!({ "harness": harness })),
            CliTelemetryEvent::RunMessageList { harness } => Some(json!({ "harness": harness })),
            CliTelemetryEvent::RunMessageRead { harness } => Some(json!({ "harness": harness })),
            CliTelemetryEvent::RunMessageMarkDelivered { harness } => {
                Some(json!({ "harness": harness }))
            }
            CliTelemetryEvent::Login => None,
            CliTelemetryEvent::Logout => None,
            CliTelemetryEvent::Whoami => None,
            CliTelemetryEvent::ProviderSetup => None,
            CliTelemetryEvent::ProviderList => None,
            CliTelemetryEvent::IntegrationCreate => None,
            CliTelemetryEvent::IntegrationUpdate => None,
            CliTelemetryEvent::IntegrationList => None,
            CliTelemetryEvent::ArtifactUpload => None,
            CliTelemetryEvent::ArtifactGet => None,
            CliTelemetryEvent::ArtifactDownload => None,
            CliTelemetryEvent::ApiKeyList => None,
            CliTelemetryEvent::ApiKeyCreate => None,
            CliTelemetryEvent::ApiKeyExpire => None,
            CliTelemetryEvent::ScheduleCreate => None,
            CliTelemetryEvent::ScheduleList => None,
            CliTelemetryEvent::ScheduleGet => None,
            CliTelemetryEvent::SchedulePause => None,
            CliTelemetryEvent::ScheduleUnpause => None,
            CliTelemetryEvent::ScheduleUpdate => None,
            CliTelemetryEvent::ScheduleDelete => None,
            CliTelemetryEvent::SecretCreate => None,
            CliTelemetryEvent::SecretDelete => None,
            CliTelemetryEvent::SecretUpdate => None,
            CliTelemetryEvent::SecretList => None,
            CliTelemetryEvent::FederateIssueToken => None,
            CliTelemetryEvent::FederateIssueGcpToken => None,
            CliTelemetryEvent::HarnessSupportPing => None,
            CliTelemetryEvent::HarnessSupportReportArtifact { artifact_type } => {
                Some(json!({ "artifact_type": artifact_type }))
            }
            CliTelemetryEvent::HarnessSupportNotifyUser => None,
            CliTelemetryEvent::HarnessSupportFinishTask { success } => {
                Some(json!({ "success": success }))
            }
            CliTelemetryEvent::HarnessSupportReportShutdown => None,
            CliTelemetryEvent::RunnerList => None,
            CliTelemetryEvent::RunnerCreate => None,
            CliTelemetryEvent::RunnerUpdate => None,
            CliTelemetryEvent::RunnerDelete => None,
        }
    }

    fn description(&self) -> &'static str {
        CliTelemetryEventDiscriminants::from(self).description()
    }

    fn enablement_state(&self) -> EnablementState {
        CliTelemetryEventDiscriminants::from(self).enablement_state()
    }

    fn contains_ugc(&self) -> bool {
        false
    }

    fn event_descs() -> impl Iterator<Item = Box<dyn TelemetryEventDesc>> {
        brasa_core::telemetry::enum_events::<Self>()
    }
}

impl TelemetryEventDesc for CliTelemetryEventDiscriminants {
    fn name(&self) -> &'static str {
        match self {
            CliTelemetryEventDiscriminants::AgentRun => "CLI.Execute.Agent.Run",
            CliTelemetryEventDiscriminants::AgentRunAmbient => "CLI.Execute.Agent.RunAmbient",
            CliTelemetryEventDiscriminants::AgentRunCloudStandby => {
                "CLI.Execute.Agent.RunCloudStandby"
            }
            CliTelemetryEventDiscriminants::AgentProfileList => "CLI.Execute.Agent.Profile.List",
            CliTelemetryEventDiscriminants::AgentList => "CLI.Execute.Agent.List",
            CliTelemetryEventDiscriminants::AgentGet => "CLI.Execute.Agent.Get",
            CliTelemetryEventDiscriminants::AgentCreate => "CLI.Execute.Agent.Create",
            CliTelemetryEventDiscriminants::AgentUpdate => "CLI.Execute.Agent.Update",
            CliTelemetryEventDiscriminants::AgentDelete => "CLI.Execute.Agent.Delete",
            CliTelemetryEventDiscriminants::AgentSkills => "CLI.Execute.Agent.Skills",
            CliTelemetryEventDiscriminants::EnvironmentList => "CLI.Execute.Environment.List",
            CliTelemetryEventDiscriminants::EnvironmentCreate => "CLI.Execute.Environment.Create",
            CliTelemetryEventDiscriminants::EnvironmentDelete => "CLI.Execute.Environment.Delete",
            CliTelemetryEventDiscriminants::EnvironmentUpdate => "CLI.Execute.Environment.Update",
            CliTelemetryEventDiscriminants::EnvironmentGet => "CLI.Execute.Environment.Get",
            CliTelemetryEventDiscriminants::EnvironmentImageList => {
                "CLI.Execute.Environment.Image.List"
            }
            CliTelemetryEventDiscriminants::EnvironmentCheckout => {
                "CLI.Execute.EnvironmentCheckout"
            }
            CliTelemetryEventDiscriminants::MCPList => "CLI.Execute.MCP.List",
            CliTelemetryEventDiscriminants::ModelList => "CLI.Execute.Model.List",
            CliTelemetryEventDiscriminants::MemoryStoreList => "CLI.Execute.MemoryStore.List",
            CliTelemetryEventDiscriminants::MemoryStoreListMemories => {
                "CLI.Execute.MemoryStore.ListMemories"
            }
            CliTelemetryEventDiscriminants::MemoryStoreCreateMemory => {
                "CLI.Execute.MemoryStore.CreateMemory"
            }
            CliTelemetryEventDiscriminants::MemoryStoreUpdateMemory => {
                "CLI.Execute.MemoryStore.UpdateMemory"
            }
            CliTelemetryEventDiscriminants::MemoryStoreDeleteMemory => {
                "CLI.Execute.MemoryStore.DeleteMemory"
            }
            CliTelemetryEventDiscriminants::MemoryStoreGetStore => {
                "CLI.Execute.MemoryStore.GetStore"
            }
            CliTelemetryEventDiscriminants::MemoryStoreUpdateStore => {
                "CLI.Execute.MemoryStore.UpdateStore"
            }
            CliTelemetryEventDiscriminants::MemoryStoreListStoreAgents => {
                "CLI.Execute.MemoryStore.ListStoreAgents"
            }
            CliTelemetryEventDiscriminants::MemoryStoreListVersions => {
                "CLI.Execute.MemoryStore.ListVersions"
            }
            CliTelemetryEventDiscriminants::TaskList => "CLI.Execute.Task.List",
            CliTelemetryEventDiscriminants::TaskGet => "CLI.Execute.Task.Get",
            CliTelemetryEventDiscriminants::ConversationGet => "CLI.Execute.Conversation.Get",
            CliTelemetryEventDiscriminants::RunConversationGet => {
                "CLI.Execute.Run.Conversation.Get"
            }
            CliTelemetryEventDiscriminants::RunMessageWatch => "CLI.Execute.Run.Message.Watch",
            CliTelemetryEventDiscriminants::RunMessageSend => "CLI.Execute.Run.Message.Send",
            CliTelemetryEventDiscriminants::RunMessageList => "CLI.Execute.Run.Message.List",
            CliTelemetryEventDiscriminants::RunMessageRead => "CLI.Execute.Run.Message.Read",
            CliTelemetryEventDiscriminants::RunMessageMarkDelivered => {
                "CLI.Execute.Run.Message.MarkDelivered"
            }
            CliTelemetryEventDiscriminants::Login => "CLI.Execute.Login",
            CliTelemetryEventDiscriminants::Logout => "CLI.Execute.Logout",
            CliTelemetryEventDiscriminants::Whoami => "CLI.Execute.Whoami",
            CliTelemetryEventDiscriminants::ProviderSetup => "CLI.Execute.Provider.Setup",
            CliTelemetryEventDiscriminants::ProviderList => "CLI.Execute.Provider.List",
            CliTelemetryEventDiscriminants::IntegrationCreate => "CLI.Execute.Integration.Create",
            CliTelemetryEventDiscriminants::IntegrationUpdate => "CLI.Execute.Integration.Update",
            CliTelemetryEventDiscriminants::IntegrationList => "CLI.Execute.Integration.List",
            CliTelemetryEventDiscriminants::ArtifactUpload => "CLI.Execute.Artifact.Upload",
            CliTelemetryEventDiscriminants::ArtifactGet => "CLI.Execute.Artifact.Get",
            CliTelemetryEventDiscriminants::ArtifactDownload => "CLI.Execute.Artifact.Download",
            CliTelemetryEventDiscriminants::ApiKeyList => "CLI.Execute.ApiKey.List",
            CliTelemetryEventDiscriminants::ApiKeyCreate => "CLI.Execute.ApiKey.Create",
            CliTelemetryEventDiscriminants::ApiKeyExpire => "CLI.Execute.ApiKey.Expire",
            CliTelemetryEventDiscriminants::ScheduleCreate => "CLI.Execute.Schedule.Create",
            CliTelemetryEventDiscriminants::ScheduleList => "CLI.Execute.Schedule.List",
            CliTelemetryEventDiscriminants::ScheduleGet => "CLI.Execute.Schedule.Get",
            CliTelemetryEventDiscriminants::SchedulePause => "CLI.Execute.Schedule.Pause",
            CliTelemetryEventDiscriminants::ScheduleUnpause => "CLI.Execute.Schedule.Unpause",
            CliTelemetryEventDiscriminants::ScheduleUpdate => "CLI.Execute.Schedule.Update",
            CliTelemetryEventDiscriminants::ScheduleDelete => "CLI.Execute.Schedule.Delete",
            CliTelemetryEventDiscriminants::SecretCreate => "CLI.Execute.Secret.Create",
            CliTelemetryEventDiscriminants::SecretDelete => "CLI.Execute.Secret.Delete",
            CliTelemetryEventDiscriminants::SecretUpdate => "CLI.Execute.Secret.Update",
            CliTelemetryEventDiscriminants::SecretList => "CLI.Execute.Secret.List",
            CliTelemetryEventDiscriminants::FederateIssueToken => "CLI.Execute.Federate.IssueToken",
            CliTelemetryEventDiscriminants::FederateIssueGcpToken => {
                "CLI.Execute.Federate.IssueGcpToken"
            }
            CliTelemetryEventDiscriminants::HarnessSupportPing => "CLI.Execute.HarnessSupport.Ping",
            CliTelemetryEventDiscriminants::HarnessSupportReportArtifact => {
                "CLI.Execute.HarnessSupport.ReportArtifact"
            }
            CliTelemetryEventDiscriminants::HarnessSupportNotifyUser => {
                "CLI.Execute.HarnessSupport.NotifyUser"
            }
            CliTelemetryEventDiscriminants::HarnessSupportFinishTask => {
                "CLI.Execute.HarnessSupport.FinishTask"
            }
            CliTelemetryEventDiscriminants::HarnessSupportReportShutdown => {
                "CLI.Execute.HarnessSupport.ReportShutdown"
            }
            CliTelemetryEventDiscriminants::RunnerList => "CLI.Execute.Runner.List",
            CliTelemetryEventDiscriminants::RunnerCreate => "CLI.Execute.Runner.Create",
            CliTelemetryEventDiscriminants::RunnerUpdate => "CLI.Execute.Runner.Update",
            CliTelemetryEventDiscriminants::RunnerDelete => "CLI.Execute.Runner.Delete",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            CliTelemetryEventDiscriminants::AgentRun => "Ran an agent from the Brasa CLI",
            CliTelemetryEventDiscriminants::AgentRunAmbient => {
                "Ran an ambient agent from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::AgentRunCloudStandby => {
                "Prepared a pre-warmed cloud sandbox from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::AgentProfileList => {
                "Listed agent profiles from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::AgentList => "Listed agents from the Brasa CLI",
            CliTelemetryEventDiscriminants::AgentGet => "Got agent details from the Brasa CLI",
            CliTelemetryEventDiscriminants::AgentCreate => "Created an agent from the Brasa CLI",
            CliTelemetryEventDiscriminants::AgentUpdate => "Updated an agent from the Brasa CLI",
            CliTelemetryEventDiscriminants::AgentDelete => "Deleted an agent from the Brasa CLI",
            CliTelemetryEventDiscriminants::AgentSkills => "Listed agent skills from the Brasa CLI",
            CliTelemetryEventDiscriminants::EnvironmentList => {
                "Listed cloud environments from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::EnvironmentCreate => {
                "Created a cloud environment from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::EnvironmentDelete => {
                "Deleted a cloud environment from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::EnvironmentUpdate => {
                "Updated a cloud environment from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::EnvironmentGet => {
                "Got cloud environment details from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::EnvironmentImageList => {
                "Listed available base images from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::EnvironmentCheckout => {
                "Prepared structured environment checkouts"
            }
            CliTelemetryEventDiscriminants::MCPList => "Listed MCP servers from the Brasa CLI",
            CliTelemetryEventDiscriminants::ModelList => "Listed models from the Brasa CLI",
            CliTelemetryEventDiscriminants::MemoryStoreList => {
                "Listed memory stores from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::MemoryStoreListMemories => {
                "Listed memories in a memory store from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::MemoryStoreCreateMemory => {
                "Created a manual memory in a memory store from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::MemoryStoreUpdateMemory => {
                "Updated a memory in a memory store from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::MemoryStoreDeleteMemory => {
                "Deleted a memory from a memory store from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::MemoryStoreGetStore => {
                "Got a memory store from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::MemoryStoreUpdateStore => {
                "Updated a memory store from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::MemoryStoreListStoreAgents => {
                "Listed agents attached to a memory store from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::MemoryStoreListVersions => {
                "Listed version history of a memory from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::TaskList => "Listed tasks from the Brasa CLI",
            CliTelemetryEventDiscriminants::TaskGet => "Got status of task from the Brasa CLI",
            CliTelemetryEventDiscriminants::ConversationGet => {
                "Got conversation by ID from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::RunConversationGet => {
                "Got run conversation from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::RunMessageWatch => {
                "Watched run messages from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::RunMessageSend => {
                "Sent a run message from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::RunMessageList => {
                "Listed run messages from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::RunMessageRead => {
                "Read a run message from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::RunMessageMarkDelivered => {
                "Marked a run message as delivered from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::Login => "Logged in via the Brasa CLI",
            CliTelemetryEventDiscriminants::Logout => "Logged out via the Brasa CLI",
            CliTelemetryEventDiscriminants::Whoami => {
                "Printed current user info from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ProviderSetup => "Set up a provider via the Brasa CLI",
            CliTelemetryEventDiscriminants::ProviderList => "Listed providers from the Brasa CLI",
            CliTelemetryEventDiscriminants::IntegrationCreate => {
                "Created an integration from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::IntegrationUpdate => {
                "Updated an integration from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::IntegrationList => {
                "Listed integrations from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ArtifactUpload => {
                "Uploaded an artifact from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ArtifactGet => {
                "Got artifact metadata from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ArtifactDownload => {
                "Downloaded an artifact from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ApiKeyList => "Listed API keys from the Brasa CLI",
            CliTelemetryEventDiscriminants::ApiKeyCreate => "Created an API key from the Brasa CLI",
            CliTelemetryEventDiscriminants::ApiKeyExpire => "Expired an API key from the Brasa CLI",
            CliTelemetryEventDiscriminants::ScheduleCreate => {
                "Created a scheduled agent from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ScheduleList => {
                "Listed scheduled agents from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ScheduleGet => {
                "Got scheduled agent configuration from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::SchedulePause => {
                "Paused a scheduled agent from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ScheduleUnpause => {
                "Unpaused a scheduled agent from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ScheduleUpdate => {
                "Updated a scheduled agent from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::ScheduleDelete => {
                "Deleted a scheduled agent from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::SecretCreate => "Created a secret from the Brasa CLI",
            CliTelemetryEventDiscriminants::SecretDelete => "Deleted a secret from the Brasa CLI",
            CliTelemetryEventDiscriminants::SecretUpdate => "Updated a secret from the Brasa CLI",
            CliTelemetryEventDiscriminants::SecretList => "Listed secrets from the Brasa CLI",
            CliTelemetryEventDiscriminants::FederateIssueToken => {
                "Issued a federated identity token from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::FederateIssueGcpToken => {
                "Issued a GCP federated identity token from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::HarnessSupportPing => {
                "Pinged harness-support from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::HarnessSupportReportArtifact => {
                "Reported an artifact via harness-support from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::HarnessSupportNotifyUser => {
                "Sent a user notification via harness-support from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::HarnessSupportFinishTask => {
                "Reported task completion via harness-support from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::HarnessSupportReportShutdown => {
                "Reported agent shutdown via harness-support from the Brasa CLI"
            }
            CliTelemetryEventDiscriminants::RunnerList => "Listed runners from the Brasa CLI",
            CliTelemetryEventDiscriminants::RunnerCreate => "Created a runner from the Brasa CLI",
            CliTelemetryEventDiscriminants::RunnerUpdate => "Updated a runner from the Brasa CLI",
            CliTelemetryEventDiscriminants::RunnerDelete => "Deleted a runner from the Brasa CLI",
        }
    }

    fn enablement_state(&self) -> EnablementState {
        match self {
            Self::FederateIssueToken | Self::FederateIssueGcpToken => {
                EnablementState::Flag(FeatureFlag::OzIdentityFederation)
            }
            Self::HarnessSupportPing
            | Self::HarnessSupportReportArtifact
            | Self::HarnessSupportNotifyUser
            | Self::HarnessSupportFinishTask => EnablementState::Flag(FeatureFlag::AgentHarness),
            Self::ArtifactUpload | Self::ArtifactGet | Self::ArtifactDownload => {
                EnablementState::Flag(FeatureFlag::ArtifactCommand)
            }
            Self::ApiKeyList | Self::ApiKeyCreate | Self::ApiKeyExpire => {
                EnablementState::Flag(FeatureFlag::APIKeyManagement)
            }
            Self::RunnerList | Self::RunnerCreate | Self::RunnerUpdate | Self::RunnerDelete => {
                EnablementState::Flag(FeatureFlag::CloudAgentRunners)
            }
            Self::RunMessageWatch
            | Self::RunMessageSend
            | Self::RunMessageList
            | Self::RunMessageRead
            | Self::RunMessageMarkDelivered => EnablementState::Always,
            _ => EnablementState::Always,
        }
    }
}

brasa_core::register_telemetry_event!(CliTelemetryEvent);
