use crate::{
    DomainTopology, ExchangeAxis, ExchangeDirection, HaloExchangeError, HaloExchangeResult,
    HaloTransfer, HorizontalBounds, HorizontalPeriodicity, HorizontalStaggering, IndexRange,
    PatchBounds,
};

pub(super) fn validate_widths(
    topology: &DomainTopology,
    halo_width: usize,
    periodicity: HorizontalPeriodicity,
    staggering: HorizontalStaggering,
) -> HaloExchangeResult<()> {
    if halo_width == 0 {
        return Err(HaloExchangeError::ZeroHaloWidth);
    }
    if halo_width > topology.maximum_halo_width() {
        return Err(HaloExchangeError::HaloWidthExceedsAllocation {
            halo_width,
            maximum_halo_width: topology.maximum_halo_width(),
        });
    }
    validate_periodic_storage(
        periodicity.west_east(),
        halo_width + usize::from(staggering.west_east()),
        topology.boundary_widths().west_east(),
    )?;
    validate_periodic_storage(
        periodicity.south_north(),
        halo_width + usize::from(staggering.south_north()),
        topology.boundary_widths().south_north(),
    )?;
    Ok(())
}

fn validate_periodic_storage(
    is_periodic: bool,
    required_width: usize,
    boundary_width: usize,
) -> HaloExchangeResult<()> {
    if is_periodic && required_width > boundary_width {
        return Err(HaloExchangeError::PeriodicBoundaryStorageTooNarrow {
            halo_width: required_width,
            boundary_width,
        });
    }
    Ok(())
}

pub(super) fn create_axis_transfers(
    topology: &DomainTopology,
    width: i32,
    axis: ExchangeAxis,
    is_periodic: bool,
    is_staggered: bool,
) -> HaloExchangeResult<Vec<HaloTransfer>> {
    let mut transfers = Vec::new();
    for destination in topology.patches() {
        if let Some(source) = lower_neighbor(topology, *destination, axis) {
            transfers.push(create_internal_transfer(
                topology,
                source,
                *destination,
                axis,
                ExchangeDirection::Lower,
                width,
            )?);
        } else if is_periodic && axis_process_count(topology, axis) > 1 {
            let source = wrapped_neighbor(topology, *destination, axis, ExchangeDirection::Lower)?;
            transfers.push(create_periodic_transfer(
                source,
                *destination,
                axis,
                ExchangeDirection::Lower,
                width,
                is_staggered,
            )?);
        }

        if let Some(source) = upper_neighbor(topology, *destination, axis) {
            transfers.push(create_internal_transfer(
                topology,
                source,
                *destination,
                axis,
                ExchangeDirection::Upper,
                width,
            )?);
        } else if is_periodic && axis_process_count(topology, axis) > 1 {
            let source = wrapped_neighbor(topology, *destination, axis, ExchangeDirection::Upper)?;
            transfers.push(create_periodic_transfer(
                source,
                *destination,
                axis,
                ExchangeDirection::Upper,
                width,
                is_staggered,
            )?);
        }
    }
    Ok(transfers)
}

fn lower_neighbor(
    topology: &DomainTopology,
    patch: PatchBounds,
    axis: ExchangeAxis,
) -> Option<PatchBounds> {
    let coordinate = patch.coordinate();
    let patch_id = match axis {
        ExchangeAxis::WestEast if coordinate.column() > 0 => {
            topology.patch_id_at(coordinate.column() - 1, coordinate.row())
        }
        ExchangeAxis::SouthNorth if coordinate.row() > 0 => {
            topology.patch_id_at(coordinate.column(), coordinate.row() - 1)
        }
        _ => None,
    }?;
    topology.patch(patch_id).ok()
}

fn upper_neighbor(
    topology: &DomainTopology,
    patch: PatchBounds,
    axis: ExchangeAxis,
) -> Option<PatchBounds> {
    let coordinate = patch.coordinate();
    let patch_id = match axis {
        ExchangeAxis::WestEast => topology.patch_id_at(coordinate.column() + 1, coordinate.row()),
        ExchangeAxis::SouthNorth => topology.patch_id_at(coordinate.column(), coordinate.row() + 1),
    }?;
    topology.patch(patch_id).ok()
}

fn wrapped_neighbor(
    topology: &DomainTopology,
    patch: PatchBounds,
    axis: ExchangeAxis,
    direction: ExchangeDirection,
) -> HaloExchangeResult<PatchBounds> {
    let coordinate = patch.coordinate();
    let patch_id = match (axis, direction) {
        (ExchangeAxis::WestEast, ExchangeDirection::Lower) => {
            topology.patch_id_at(topology.process_grid().columns() - 1, coordinate.row())
        }
        (ExchangeAxis::WestEast, ExchangeDirection::Upper) => {
            topology.patch_id_at(0, coordinate.row())
        }
        (ExchangeAxis::SouthNorth, ExchangeDirection::Lower) => {
            topology.patch_id_at(coordinate.column(), topology.process_grid().rows() - 1)
        }
        (ExchangeAxis::SouthNorth, ExchangeDirection::Upper) => {
            topology.patch_id_at(coordinate.column(), 0)
        }
    }
    .ok_or(HaloExchangeError::TopologyInconsistent)?;
    topology
        .patch(patch_id)
        .map_err(|_| HaloExchangeError::TopologyInconsistent)
}

fn axis_process_count(topology: &DomainTopology, axis: ExchangeAxis) -> usize {
    match axis {
        ExchangeAxis::WestEast => topology.process_grid().columns(),
        ExchangeAxis::SouthNorth => topology.process_grid().rows(),
    }
}

fn create_internal_transfer(
    topology: &DomainTopology,
    source_patch: PatchBounds,
    destination_patch: PatchBounds,
    axis: ExchangeAxis,
    direction: ExchangeDirection,
    width: i32,
) -> HaloExchangeResult<HaloTransfer> {
    let destination_owned = destination_patch.owned();
    let domain = topology.domain().horizontal();
    let destination_axis = match (axis, direction) {
        (ExchangeAxis::WestEast, ExchangeDirection::Lower) => checked_range(
            destination_owned.west_east().start().checked_sub(width),
            Some(destination_owned.west_east().start()),
        )?,
        (ExchangeAxis::WestEast, ExchangeDirection::Upper) => checked_range(
            Some(destination_owned.west_east().end()),
            destination_owned.west_east().end().checked_add(width),
        )?,
        (ExchangeAxis::SouthNorth, ExchangeDirection::Lower) => checked_range(
            destination_owned.south_north().start().checked_sub(width),
            Some(destination_owned.south_north().start()),
        )?,
        (ExchangeAxis::SouthNorth, ExchangeDirection::Upper) => checked_range(
            Some(destination_owned.south_north().end()),
            destination_owned.south_north().end().checked_add(width),
        )?,
    };
    let transverse = clipped_transverse(destination_owned, domain, axis, width)?;
    let destination = combine_axes(axis, destination_axis, transverse);
    create_validated_transfer(
        source_patch,
        destination_patch,
        axis,
        direction,
        destination,
        destination,
    )
}

fn create_periodic_transfer(
    source_patch: PatchBounds,
    destination_patch: PatchBounds,
    axis: ExchangeAxis,
    direction: ExchangeDirection,
    width: i32,
    is_staggered: bool,
) -> HaloExchangeResult<HaloTransfer> {
    let source_owned = source_patch.owned();
    let destination_owned = destination_patch.owned();
    let stagger = i32::from(is_staggered);
    let (source_axis, destination_axis) = match (axis, direction) {
        (ExchangeAxis::WestEast, ExchangeDirection::Lower) => (
            checked_range(
                source_owned
                    .west_east()
                    .end()
                    .checked_sub(1)
                    .and_then(|end| end.checked_sub(width)),
                source_owned.west_east().end().checked_sub(1),
            )?,
            checked_range(
                destination_owned.west_east().start().checked_sub(width),
                Some(destination_owned.west_east().start()),
            )?,
        ),
        (ExchangeAxis::WestEast, ExchangeDirection::Upper) => (
            checked_range(
                Some(source_owned.west_east().start()),
                source_owned
                    .west_east()
                    .start()
                    .checked_add(width)
                    .and_then(|end| end.checked_add(stagger)),
            )?,
            checked_range(
                destination_owned.west_east().end().checked_sub(1),
                destination_owned
                    .west_east()
                    .end()
                    .checked_sub(1)
                    .and_then(|start| start.checked_add(width))
                    .and_then(|end| end.checked_add(stagger)),
            )?,
        ),
        (ExchangeAxis::SouthNorth, ExchangeDirection::Lower) => (
            checked_range(
                source_owned
                    .south_north()
                    .end()
                    .checked_sub(1)
                    .and_then(|end| end.checked_sub(width)),
                source_owned.south_north().end().checked_sub(1),
            )?,
            checked_range(
                destination_owned.south_north().start().checked_sub(width),
                Some(destination_owned.south_north().start()),
            )?,
        ),
        (ExchangeAxis::SouthNorth, ExchangeDirection::Upper) => (
            checked_range(
                Some(source_owned.south_north().start()),
                source_owned
                    .south_north()
                    .start()
                    .checked_add(width)
                    .and_then(|end| end.checked_add(stagger)),
            )?,
            checked_range(
                destination_owned.south_north().end().checked_sub(1),
                destination_owned
                    .south_north()
                    .end()
                    .checked_sub(1)
                    .and_then(|start| start.checked_add(width))
                    .and_then(|end| end.checked_add(stagger)),
            )?,
        ),
    };
    let source_transverse = expanded_transverse(source_owned, axis, width)?;
    let destination_transverse = expanded_transverse(destination_owned, axis, width)?;
    create_validated_transfer(
        source_patch,
        destination_patch,
        axis,
        direction,
        combine_axes(axis, source_axis, source_transverse),
        combine_axes(axis, destination_axis, destination_transverse),
    )
}

fn clipped_transverse(
    owned: HorizontalBounds,
    domain: HorizontalBounds,
    axis: ExchangeAxis,
    width: i32,
) -> HaloExchangeResult<IndexRange> {
    let (owned_range, domain_range) = match axis {
        ExchangeAxis::WestEast => (owned.south_north(), domain.south_north()),
        ExchangeAxis::SouthNorth => (owned.west_east(), domain.west_east()),
    };
    let start = owned_range
        .start()
        .checked_sub(width)
        .ok_or(HaloExchangeError::IndexArithmeticOverflow)?
        .max(domain_range.start());
    let end = owned_range
        .end()
        .checked_add(width)
        .ok_or(HaloExchangeError::IndexArithmeticOverflow)?
        .min(domain_range.end());
    Ok(IndexRange::from_validated(start, end))
}

fn expanded_transverse(
    owned: HorizontalBounds,
    axis: ExchangeAxis,
    width: i32,
) -> HaloExchangeResult<IndexRange> {
    let owned_range = match axis {
        ExchangeAxis::WestEast => owned.south_north(),
        ExchangeAxis::SouthNorth => owned.west_east(),
    };
    checked_range(
        owned_range.start().checked_sub(width),
        owned_range.end().checked_add(width),
    )
}

fn combine_axes(
    axis: ExchangeAxis,
    exchanged: IndexRange,
    transverse: IndexRange,
) -> HorizontalBounds {
    match axis {
        ExchangeAxis::WestEast => HorizontalBounds::new(exchanged, transverse),
        ExchangeAxis::SouthNorth => HorizontalBounds::new(transverse, exchanged),
    }
}

fn checked_range(start: Option<i32>, end: Option<i32>) -> HaloExchangeResult<IndexRange> {
    let start = start.ok_or(HaloExchangeError::IndexArithmeticOverflow)?;
    let end = end.ok_or(HaloExchangeError::IndexArithmeticOverflow)?;
    if start >= end {
        return Err(HaloExchangeError::EmptyTransferRange { start, end });
    }
    Ok(IndexRange::from_validated(start, end))
}

fn create_validated_transfer(
    source_patch: PatchBounds,
    destination_patch: PatchBounds,
    axis: ExchangeAxis,
    direction: ExchangeDirection,
    source: HorizontalBounds,
    destination: HorizontalBounds,
) -> HaloExchangeResult<HaloTransfer> {
    if !source_patch.memory().horizontal().contains(source) {
        return Err(HaloExchangeError::TransferOutsideMemory {
            patch_id: source_patch.patch_id(),
        });
    }
    if !destination_patch
        .memory()
        .horizontal()
        .contains(destination)
    {
        return Err(HaloExchangeError::TransferOutsideMemory {
            patch_id: destination_patch.patch_id(),
        });
    }
    if source.west_east().len() != destination.west_east().len()
        || source.south_north().len() != destination.south_north().len()
    {
        return Err(HaloExchangeError::TransferShapeMismatch);
    }
    Ok(HaloTransfer::new(
        source_patch.patch_id(),
        destination_patch.patch_id(),
        axis,
        direction,
        source,
        destination,
        source_patch.memory().bottom_top(),
    ))
}
