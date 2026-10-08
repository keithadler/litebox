// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! The kernel's broker core: hardware randomness, plus whatever file service
//! the service needs.

use alloc::sync::Arc;
use litebox_broker_core::{
    BrokerCore, ObjectRights, PolicyEngine,
    fs::FileService,
    random::{RandomProvider, RandomProviderError},
    socket::UnsupportedSocketProvider,
    timer::UnsupportedTimerProvider,
};

/// The HAL's hardware CSPRNG as the broker's randomness.
pub struct HardwareRandom(litebox_hal::rng::Hardware);

impl HardwareRandom {
    /// # Panics
    ///
    /// Without a hardware CSPRNG.
    pub fn new() -> Self {
        Self(litebox_hal::rng::hardware())
    }
}

impl RandomProvider for HardwareRandom {
    fn fill(&self, output: &mut [u8]) -> Result<(), RandomProviderError> {
        self.0.fill(output).map_err(|_| RandomProviderError)
    }
}

/// At most once: only one broker core may exist. Runner processes, which the
/// kernel vouches for, get `rights` to the objects they open.
///
/// # Panics
///
/// Without a hardware CSPRNG, or on a second call.
pub fn core(fs: Arc<dyn FileService>, rights: ObjectRights) -> BrokerCore {
    BrokerCore::new(
        PolicyEngine::with_host_guaranteed_rights(rights),
        Arc::new(UnsupportedSocketProvider),
        Arc::new(HardwareRandom::new()),
        Arc::new(UnsupportedTimerProvider),
        fs,
    )
    .unwrap_or_else(|error| panic!("broker core: {error:?}"))
}
