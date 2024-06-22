#[derive(Debug, Clone)]
pub enum AeronState {
    Healthy(HealthyState),
    Unhealthy(UnhealthyStates),
    Uninitialized,
    Inactive(InactiveState),
    Error(ErrorState),
}

impl AeronState {
    pub fn default() -> Self {
        AeronState::Uninitialized
    }
}

#[derive(Debug, Clone)]
pub enum ActiveState {
    Healthy(HealthyState),
    Unhealthy(UnhealthyStates),
}

#[derive(Debug, Clone)]
pub enum HealthyState {
    Active,
    Connecting,
    Reconnecting,
    Acknowledged,
    FlowControl,
}

#[derive(Debug, Clone)]
pub enum UnhealthyStates {
    SlowConsumer,
    HighLatency,
    BufferOverflow,
    DroppedMessages,
    ErrorRateThreshold,
    Degraded,
}

#[derive(Debug, Clone)]
pub enum InactiveState {
    Idle,
    Paused,
    Draining,
    TaskComplete,
    Unresponsive,
    AwaitingResources,
}

#[derive(Debug, Clone)]
pub enum ErrorState {
    Disconnected,
    Timeout,
    ConfigError,
    TransportError,
    ProtocolError,
    AuthError,
    PermissionDenied,
    ResourcesExhausted,
}
