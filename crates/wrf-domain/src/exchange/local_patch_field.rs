use crate::{
    DomainTopology, HaloExchangeError, HaloExchangeResult, HaloTransfer, PatchField, PatchId,
};

/// Contiguous host storage for every patch in a local topology simulation.
///
/// Each patch uses WRF's XZY order: west-east is contiguous, followed by
/// bottom-top, then south-north. This type is the deterministic reference
/// storage used to validate transports; numerical kernels remain free to use
/// their backend-native field types.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalPatchField<Value> {
    topology: DomainTopology,
    patches: Vec<PatchField<Value>>,
}

impl<Value> LocalPatchField<Value>
where
    Value: Clone,
{
    /// Allocates every patch memory region with one initial value.
    pub fn try_from_value(
        topology: DomainTopology,
        initial_value: Value,
    ) -> HaloExchangeResult<Self> {
        let patches = topology
            .patches()
            .iter()
            .map(|bounds| PatchField::try_from_value(*bounds, initial_value.clone()))
            .collect::<HaloExchangeResult<Vec<_>>>()?;
        Ok(Self { topology, patches })
    }

    /// Returns the topology that owns this field.
    pub const fn topology(&self) -> &DomainTopology {
        &self.topology
    }

    /// Returns one value from allocated patch memory.
    pub fn value(
        &self,
        patch_id: PatchId,
        west_east: i32,
        bottom_top: i32,
        south_north: i32,
    ) -> HaloExchangeResult<&Value> {
        self.patch_storage(patch_id)?
            .value(west_east, bottom_top, south_north)
    }

    /// Sets one value in allocated patch memory.
    pub fn set_value(
        &mut self,
        patch_id: PatchId,
        west_east: i32,
        bottom_top: i32,
        south_north: i32,
        value: Value,
    ) -> HaloExchangeResult<()> {
        self.patch_storage_mut(patch_id)?
            .set_value(west_east, bottom_top, south_north, value)
    }

    pub(crate) fn pack_transfer(&self, transfer: HaloTransfer) -> HaloExchangeResult<Vec<Value>> {
        self.patch_storage(transfer.source_patch_id())?
            .pack_transfer(transfer)
    }

    pub(crate) fn unpack_transfer(
        &mut self,
        transfer: HaloTransfer,
        packed: &[Value],
    ) -> HaloExchangeResult<()> {
        self.patch_storage_mut(transfer.destination_patch_id())?
            .unpack_transfer(transfer, packed)
    }

    fn patch_storage(&self, patch_id: PatchId) -> HaloExchangeResult<&PatchField<Value>> {
        self.patches
            .get(patch_id.value())
            .ok_or(HaloExchangeError::UnknownPatch { patch_id })
    }

    fn patch_storage_mut(
        &mut self,
        patch_id: PatchId,
    ) -> HaloExchangeResult<&mut PatchField<Value>> {
        self.patches
            .get_mut(patch_id.value())
            .ok_or(HaloExchangeError::UnknownPatch { patch_id })
    }
}

#[cfg(test)]
mod tests {
    use crate::{BoundaryWidths, DomainBounds, IndexRange, ProcessGrid};

    use super::*;

    #[test]
    fn field_uses_xzy_memory_order_without_exposing_raw_layout() {
        let domain = DomainBounds::new(
            IndexRange::try_new(0, 2).unwrap(),
            IndexRange::try_new(0, 2).unwrap(),
            IndexRange::try_new(0, 2).unwrap(),
        );
        let topology = DomainTopology::try_new(
            domain,
            ProcessGrid::try_new(1, 1).unwrap(),
            0,
            BoundaryWidths::default(),
        )
        .unwrap();
        let patch_id = topology.patches()[0].patch_id();
        let mut field = LocalPatchField::try_from_value(topology, 0_i32).unwrap();

        field.set_value(patch_id, 1, 1, 1, 42).unwrap();

        assert_eq!(*field.value(patch_id, 1, 1, 1).unwrap(), 42);
        assert_eq!(*field.value(patch_id, 0, 1, 1).unwrap(), 0);
    }
}
