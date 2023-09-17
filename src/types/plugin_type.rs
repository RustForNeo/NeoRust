#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodePluginType {
	ApplicationLogs,
	CoreMetrics,
	ImportBlocks,
	LevelDbStore,
	RocksDbStore,
	RpcNep17Tracker,
	RpcSecurity,
	RpcServerPlugin,
	RpcSystemAssetTracker,
	SimplePolicy,
	StatesDumper,
	SystemLog,
}

impl NodePluginType {
	pub fn as_str(&self) -> &str {
		match self {
			Self::ApplicationLogs => "ApplicationLogs",
			Self::CoreMetrics => "CoreMetrics",
			Self::ImportBlocks => "ImportBlocks",
			Self::LevelDbStore => "LevelDBStore",
			Self::RocksDbStore => "RocksDBStore",
			Self::RpcNep17Tracker => "RpcNep17Tracker",
			Self::RpcSecurity => "RpcSecurity",
			Self::RpcServerPlugin => "RpcServerPlugin",
			Self::RpcSystemAssetTracker => "RpcSystemAssetTrackerPlugin",
			Self::SimplePolicy => "SimplePolicyPlugin",
			Self::StatesDumper => "StatesDumper",
			Self::SystemLog => "SystemLog",
		}
	}
}
