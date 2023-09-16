use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::hash::Hash;
use lazy_static::lazy_static;
use crate::utils::bytes::{BytesExtern};

lazy_static!(
    static ref INTEROP_SERVICE_HASHES: Arc<Mutex<HashMap<String, String>>> = {
        Arc::new(Mutex::new(HashMap::new()))
    };
);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InteropService {
    SystemCryptoCheckSig,
    SystemCryptoCheckMultisig,
    SystemContractCall,
    SystemContractCallNative,
    SystemContractGetCallFlags,
    SystemContractCreateStandardAccount,
    SystemContractCreateMultiSigAccount,
    SystemContractNativeOnPersist,
    SystemContractNativePostPersist,
    SystemIteratorNext,
    SystemIteratorValue,
    SystemRuntimePlatform,
    SystemRuntimeGetTrigger,
    SystemRuntimeGetTime,
    SystemRuntimeGetScriptContainer,
    SystemRuntimeGetExecutingScriptHash,
    SystemRuntimeGetCallingScriptHash,
    SystemRuntimeGetEntryScriptHash,
    SystemRuntimeCheckWitness,
    SystemRuntimeGetInvocationCounter,
    SystemRuntimeLog,
    SystemRuntimeNotify,
    SystemRuntimeGetNotifications,
    SystemRuntimeGasLeft,
    SystemRuntimeBurnGas,
    SystemRuntimeGetNetwork,
    SystemRuntimeGetRandom,
    SystemStorageGetContext,
    SystemStorageGetReadOnlyContext,
    SystemStorageAsReadOnly,
    SystemStorageGet,
    SystemStorageFind,
    SystemStoragePut,
    SystemStorageDelete,
}

impl InteropService {
    pub fn to_string(&self) -> String {
        match self {
            InteropService::SystemCryptoCheckSig => "System.Crypto.CheckSig".to_string(),
            InteropService::SystemCryptoCheckMultisig => "System.Crypto.CheckMultisig".to_string(),
            InteropService::SystemContractCall => "System.Contract.Call".to_string(),
            InteropService::SystemContractCallNative => "System.Contract.CallNative".to_string(),
            InteropService::SystemContractGetCallFlags => "System.Contract.GetCallFlags".to_string(),
            InteropService::SystemContractCreateStandardAccount => "System.Contract.CreateStandardAccount".to_string(),
            InteropService::SystemContractCreateMultiSigAccount => "System.Contract.CreateMultisigAccount".to_string(),
            InteropService::SystemContractNativeOnPersist => "System.Contract.NativeOnPersist".to_string(),
            InteropService::SystemContractNativePostPersist => "System.Contract.NativePostPersist".to_string(),
            InteropService::SystemIteratorNext => "System.Iterator.Next".to_string(),
            InteropService::SystemIteratorValue => "System.Iterator.Value".to_string(),
            InteropService::SystemRuntimePlatform => "System.Runtime.Platform".to_string(),
            InteropService::SystemRuntimeGetTrigger => "System.Runtime.GetTrigger".to_string(),
            InteropService::SystemRuntimeGetTime => "System.Runtime.GetTime".to_string(),
            InteropService::SystemRuntimeGetScriptContainer => "System.Runtime.GetScriptContainer".to_string(),
            InteropService::SystemRuntimeGetExecutingScriptHash => "System.Runtime.GetExecutingScriptHash".to_string(),
            InteropService::SystemRuntimeGetCallingScriptHash => "System.Runtime.GetCallingScriptHash".to_string(),
            InteropService::SystemRuntimeGetEntryScriptHash => "System.Runtime.GetEntryScriptHash".to_string(),
            InteropService::SystemRuntimeCheckWitness => "System.Runtime.CheckWitness".to_string(),
            InteropService::SystemRuntimeGetInvocationCounter => "System.Runtime.GetInvocationCounter".to_string(),
            InteropService::SystemRuntimeLog => "System.Runtime.Log".to_string(),
            InteropService::SystemRuntimeNotify => "System.Runtime.Notify".to_string(),
            InteropService::SystemRuntimeGetNotifications => "System.Runtime.GetNotifications".to_string(),
            InteropService::SystemRuntimeGasLeft => "System.Runtime.GasLeft".to_string(),
            InteropService::SystemRuntimeBurnGas => "System.Runtime.BurnGas".to_string(),
            InteropService::SystemRuntimeGetNetwork => "System.Runtime.GetNetwork".to_string(),
            InteropService::SystemRuntimeGetRandom => "System.Runtime.GetRandom".to_string(),
            InteropService::SystemStorageGetContext => "System.Storage.GetContext".to_string(),
            InteropService::SystemStorageGetReadOnlyContext => "System.Storage.GetReadOnlyContext".to_string(),
            InteropService::SystemStorageAsReadOnly => "System.Storage.AsReadOnly".to_string(),
            InteropService::SystemStorageGet => "System.Storage.Get".to_string(),
            InteropService::SystemStorageFind => "System.Storage.Find".to_string(),
            InteropService::SystemStoragePut => "System.Storage.Put".to_string(),
            InteropService::SystemStorageDelete => "System.Storage.Delete".to_string(),
        }
    }

    pub fn hash(&self) -> String {
        let mut hashes = INTEROP_SERVICE_HASHES.lock().unwrap();
        return if let Some(hash) = hashes.get(self.as_str()) {
            hash.clone()
        } else {
            let bytes = self.to_string().as_bytes();
            let sha = bytes.to_vec().hash256();
            let hash = hex::encode(sha)[..4].to_string();
            hashes.insert(self.to_string(), hash.clone());
            hash
        }
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