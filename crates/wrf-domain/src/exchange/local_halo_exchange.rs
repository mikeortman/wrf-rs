use crate::{HaloExchangePlan, HaloExchangeResult, LocalPatchField};

/// Deterministic in-process executor for a transport-neutral halo plan.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalHaloExchange;

impl LocalHaloExchange {
    /// Applies both WRF exchange phases using bounded message buffers.
    ///
    /// Every message in a phase is packed before any destination is mutated,
    /// matching distributed-memory semantics without cloning full fields.
    pub fn execute<Value>(
        plan: &HaloExchangePlan,
        field: &mut LocalPatchField<Value>,
    ) -> HaloExchangeResult<()>
    where
        Value: Clone,
    {
        if plan.topology() != field.topology() {
            return Err(crate::HaloExchangeError::TopologyMismatch);
        }
        for phase in plan.phases() {
            let packed_messages = phase
                .iter()
                .map(|transfer| Ok((*transfer, field.pack_transfer(*transfer)?)))
                .collect::<HaloExchangeResult<Vec<_>>>()?;
            for (transfer, packed) in packed_messages {
                field.unpack_transfer(transfer, &packed)?;
            }
        }
        Ok(())
    }
}
