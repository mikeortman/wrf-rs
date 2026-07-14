use crate::{
    HaloExchangeError, HaloExchangeResult, HorizontalBounds, IndexRange, PatchBounds, PatchId,
};

/// Contiguous XZY host storage for one process patch.
///
/// This transport buffer owns only one rank's memory. It is intentionally
/// separate from numerical backend storage so MPI can be validated without
/// forcing CPU layout or MPI types through scientific kernel interfaces.
#[derive(Clone, Debug, PartialEq)]
pub struct PatchField<Value> {
    bounds: PatchBounds,
    values: Vec<Value>,
}

impl<Value> PatchField<Value>
where
    Value: Clone,
{
    /// Allocates the patch's complete memory bounds with one initial value.
    pub fn try_from_value(bounds: PatchBounds, initial_value: Value) -> HaloExchangeResult<Self> {
        let point_count = bounds
            .memory()
            .point_count()
            .ok_or(HaloExchangeError::FieldPointCountOverflow)?;
        Ok(Self {
            bounds,
            values: vec![initial_value; point_count],
        })
    }

    /// Returns this field's patch bounds.
    pub const fn bounds(&self) -> PatchBounds {
        self.bounds
    }

    /// Returns this field's patch identifier.
    pub const fn patch_id(&self) -> PatchId {
        self.bounds.patch_id()
    }

    /// Returns one value from allocated patch memory.
    pub fn value(
        &self,
        west_east: i32,
        bottom_top: i32,
        south_north: i32,
    ) -> HaloExchangeResult<&Value> {
        let index = memory_index(self.bounds, west_east, bottom_top, south_north)?;
        Ok(&self.values[index])
    }

    /// Sets one value in allocated patch memory.
    pub fn set_value(
        &mut self,
        west_east: i32,
        bottom_top: i32,
        south_north: i32,
        value: Value,
    ) -> HaloExchangeResult<()> {
        let index = memory_index(self.bounds, west_east, bottom_top, south_north)?;
        self.values[index] = value;
        Ok(())
    }

    /// Packs this field as the source of a validated transfer.
    pub fn pack_transfer(&self, transfer: crate::HaloTransfer) -> HaloExchangeResult<Vec<Value>> {
        if transfer.source_patch_id() != self.patch_id() {
            return Err(HaloExchangeError::UnknownPatch {
                patch_id: transfer.source_patch_id(),
            });
        }
        self.pack_region(transfer.source(), transfer.bottom_top())
    }

    /// Allocates a correctly sized receive buffer for a validated transfer.
    ///
    /// Existing destination values seed the buffer so no `Default` bound is
    /// imposed on field scalars; MPI overwrites the complete buffer.
    pub fn prepare_receive_buffer(
        &self,
        transfer: crate::HaloTransfer,
    ) -> HaloExchangeResult<Vec<Value>> {
        if transfer.destination_patch_id() != self.patch_id() {
            return Err(HaloExchangeError::UnknownPatch {
                patch_id: transfer.destination_patch_id(),
            });
        }
        self.pack_region(transfer.destination(), transfer.bottom_top())
    }

    /// Unpacks a received transfer into this field's destination region.
    pub fn unpack_transfer(
        &mut self,
        transfer: crate::HaloTransfer,
        packed: &[Value],
    ) -> HaloExchangeResult<()> {
        if transfer.destination_patch_id() != self.patch_id() {
            return Err(HaloExchangeError::UnknownPatch {
                patch_id: transfer.destination_patch_id(),
            });
        }
        self.unpack_region(transfer.destination(), transfer.bottom_top(), packed)
    }

    fn pack_region(
        &self,
        horizontal: HorizontalBounds,
        bottom_top: IndexRange,
    ) -> HaloExchangeResult<Vec<Value>> {
        validate_region(self.bounds, horizontal, bottom_top)?;
        let point_count = region_point_count(horizontal, bottom_top)?;
        let mut packed = Vec::with_capacity(point_count);
        for south_north in horizontal.south_north().start()..horizontal.south_north().end() {
            for level in bottom_top.start()..bottom_top.end() {
                for west_east in horizontal.west_east().start()..horizontal.west_east().end() {
                    packed.push(self.value(west_east, level, south_north)?.clone());
                }
            }
        }
        Ok(packed)
    }

    fn unpack_region(
        &mut self,
        horizontal: HorizontalBounds,
        bottom_top: IndexRange,
        packed: &[Value],
    ) -> HaloExchangeResult<()> {
        validate_region(self.bounds, horizontal, bottom_top)?;
        let expected = region_point_count(horizontal, bottom_top)?;
        if packed.len() != expected {
            return Err(HaloExchangeError::PackedValueCountMismatch {
                expected,
                actual: packed.len(),
            });
        }
        let mut packed_index = 0;
        for south_north in horizontal.south_north().start()..horizontal.south_north().end() {
            for level in bottom_top.start()..bottom_top.end() {
                for west_east in horizontal.west_east().start()..horizontal.west_east().end() {
                    self.set_value(west_east, level, south_north, packed[packed_index].clone())?;
                    packed_index += 1;
                }
            }
        }
        Ok(())
    }
}

fn region_point_count(
    horizontal: HorizontalBounds,
    bottom_top: IndexRange,
) -> HaloExchangeResult<usize> {
    horizontal
        .west_east()
        .len()
        .checked_mul(bottom_top.len())
        .and_then(|count| count.checked_mul(horizontal.south_north().len()))
        .ok_or(HaloExchangeError::FieldPointCountOverflow)
}

fn validate_region(
    bounds: PatchBounds,
    horizontal: HorizontalBounds,
    bottom_top: IndexRange,
) -> HaloExchangeResult<()> {
    if !bounds.memory().horizontal().contains(horizontal)
        || !bounds.memory().bottom_top().contains_range(bottom_top)
    {
        return Err(HaloExchangeError::TransferOutsideMemory {
            patch_id: bounds.patch_id(),
        });
    }
    Ok(())
}

fn memory_index(
    patch: PatchBounds,
    west_east: i32,
    bottom_top: i32,
    south_north: i32,
) -> HaloExchangeResult<usize> {
    let memory = patch.memory();
    if !memory.west_east().contains(west_east)
        || !memory.bottom_top().contains(bottom_top)
        || !memory.south_north().contains(south_north)
    {
        return Err(HaloExchangeError::CoordinateOutsideMemory {
            patch_id: patch.patch_id(),
            west_east,
            bottom_top,
            south_north,
        });
    }
    let west_east_offset = (west_east - memory.west_east().start()) as usize;
    let bottom_top_offset = (bottom_top - memory.bottom_top().start()) as usize;
    let south_north_offset = (south_north - memory.south_north().start()) as usize;
    Ok(
        (south_north_offset * memory.bottom_top().len() + bottom_top_offset)
            * memory.west_east().len()
            + west_east_offset,
    )
}

#[cfg(test)]
mod tests {
    use crate::{BoundaryWidths, DomainBounds, DomainTopology, IndexRange, ProcessGrid};

    use super::*;

    #[test]
    fn patch_field_owns_only_one_patch_allocation() {
        let topology = DomainTopology::try_new(
            DomainBounds::new(
                IndexRange::try_new(0, 4).unwrap(),
                IndexRange::try_new(0, 2).unwrap(),
                IndexRange::try_new(0, 3).unwrap(),
            ),
            ProcessGrid::try_new(2, 1).unwrap(),
            1,
            BoundaryWidths::default(),
        )
        .unwrap();
        let patch = topology.patches()[0];
        let mut field = PatchField::try_from_value(patch, 0_i32).unwrap();

        field.set_value(1, 1, 2, 7).unwrap();

        assert_eq!(*field.value(1, 1, 2).unwrap(), 7);
        assert_eq!(field.values.len(), patch.memory().point_count().unwrap());
    }
}
