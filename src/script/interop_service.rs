use std::hash::Hash;
use crate::utils::bytes::{Bytes, BytesExtern};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InteropService {
    SystemCryptoCheckSig = "System.Crypto.CheckSig".parse().unwrap(),
    SystemCryptoCheckMultisig = "System.Crypto.CheckMultisig".parse().unwrap(),
    SystemContractCall = "System.Contract.Call".parse().unwrap(),
    SystemContractCallNative = "System.Contract.CallNative".parse().unwrap(),
    SystemContractGetCallFlags = "System.Contract.GetCallFlags".parse().unwrap(),
    SystemContractCreateStandardAccount = "System.Contract.CreateStandardAccount".parse().unwrap(),
    SystemContractCreateMultiSigAccount = "System.Contract.CreateMultisigAccount".parse().unwrap(),
    SystemContractNativeOnPersist = "System.Contract.NativeOnPersist".parse().unwrap(),
    SystemContractNativePostPersist = "System.Contract.NativePostPersist".parse().unwrap(),
    SystemIteratorNext = "System.Iterator.Next".parse().unwrap(),
    SystemIteratorValue = "System.Iterator.Value".parse().unwrap(),
    SystemRuntimePlatform = "System.Runtime.Platform".parse().unwrap(),
    SystemRuntimeGetTrigger = "System.Runtime.GetTrigger".parse().unwrap(),
    SystemRuntimeGetTime = "System.Runtime.GetTime".parse().unwrap(),
    SystemRuntimeGetScriptContainer = "System.Runtime.GetScriptContainer".parse().unwrap(),
    SystemRuntimeGetExecutingScriptHash = "System.Runtime.GetExecutingScriptHash".parse().unwrap(),
    SystemRuntimeGetCallingScriptHash = "System.Runtime.GetCallingScriptHash".parse().unwrap(),
    SystemRuntimeGetEntryScriptHash = "System.Runtime.GetEntryScriptHash".parse().unwrap(),
    SystemRuntimeCheckWitness = "System.Runtime.CheckWitness".parse().unwrap(),
    SystemRuntimeGetInvocationCounter = "System.Runtime.GetInvocationCounter".parse().unwrap(),
    SystemRuntimeLog = "System.Runtime.Log".parse().unwrap(),
    SystemRuntimeNotify = "System.Runtime.Notify".parse().unwrap(),
    SystemRuntimeGetNotifications = "System.Runtime.GetNotifications".parse().unwrap(),
    SystemRuntimeGasLeft = "System.Runtime.GasLeft".parse().unwrap(),
    SystemRuntimeBurnGas = "System.Runtime.BurnGas".parse().unwrap(),
    SystemRuntimeGetNetwork = "System.Runtime.GetNetwork".parse().unwrap(),
    SystemRuntimeGetRandom = "System.Runtime.GetRandom".parse().unwrap(),
    SystemStorageGetContext = "System.Storage.GetContext".parse().unwrap(),
    SystemStorageGetReadOnlyContext = "System.Storage.GetReadOnlyContext".parse().unwrap(),
    SystemStorageAsReadOnly = "System.Storage.AsReadOnly".parse().unwrap(),
    SystemStorageGet = "System.Storage.Get".parse().unwrap(),
    SystemStorageFind = "System.Storage.Find".parse().unwrap(),
    SystemStoragePut = "System.Storage.Put".parse().unwrap(),
    SystemStorageDelete = "System.Storage.Delete".parse().unwrap(),
}

impl InteropService {
    pub fn hash(&self) -> String {
        let bytes = self.to_string().as_bytes();
        let sha = bytes.to_vec().hash256();
        hex::encode(sha)[..4].to_string()
    }

    pub fn price(&self) -> u64 {
        match self {
            InteropService::SystemRuntimePlatform |
            InteropService::SystemRuntimeGetTrigger |
            InteropService::SystemRuntimeGetTime |
            InteropService::SystemRuntimeGetScriptContainer |
            InteropService::SystemRuntimeGetNetwork => 1 << 3,

            InteropService::SystemIteratorValue |
            InteropService::SystemRuntimeGetExecutingScriptHash |
            InteropService::SystemRuntimeGetCallingScriptHash |
            InteropService::SystemRuntimeGetEntryScriptHash |
            InteropService::SystemRuntimeGetInvocationCounter |
            InteropService::SystemRuntimeGasLeft |
            InteropService::SystemRuntimeBurnGas |
            InteropService::SystemRuntimeGetRandom |
            InteropService::SystemStorageGetContext |
            InteropService::SystemStorageGetReadOnlyContext |
            InteropService::SystemStorageAsReadOnly => 1 << 4,

            InteropService::SystemContractGetCallFlags |
            InteropService::SystemRuntimeCheckWitness => 1 << 10,

            InteropService::SystemRuntimeGetNotifications => 1 << 12,

            InteropService::SystemCryptoCheckSig |
            InteropService::SystemContractCall |
            InteropService::SystemContractCreateStandardAccount |
            InteropService::SystemIteratorNext |
            InteropService::SystemRuntimeLog |
            InteropService::SystemRuntimeNotify |
            InteropService::SystemStorageGet |
            InteropService::SystemStorageFind |
            InteropService::SystemStoragePut |
            InteropService::SystemStorageDelete => 1 << 15,

            _ => 0
        }
    }
}