mod plan_builder;

use crate::{
    DomainTopology, ExchangeAxis, HaloExchangeError, HaloExchangeResult, HaloTransfer,
    HorizontalPeriodicity, HorizontalStaggering,
};

use plan_builder::{create_axis_transfers, validate_widths};

/// Immutable, storage-neutral description of a complete WRF halo exchange.
///
/// The first phase is south-north. The second is west-east and includes the
/// newly populated south-north halo, which propagates corner values exactly as
/// RSL_LITE's generated Y-then-X call sequence does.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HaloExchangePlan {
    topology: DomainTopology,
    halo_width: usize,
    periodicity: HorizontalPeriodicity,
    staggering: HorizontalStaggering,
    phases: [Vec<HaloTransfer>; 2],
}

impl HaloExchangePlan {
    /// Validates storage and builds every local or transport message.
    pub fn try_new(
        topology: DomainTopology,
        halo_width: usize,
        periodicity: HorizontalPeriodicity,
        staggering: HorizontalStaggering,
    ) -> HaloExchangeResult<Self> {
        validate_widths(&topology, halo_width, periodicity, staggering)?;
        let width =
            i32::try_from(halo_width).map_err(|_| HaloExchangeError::IndexArithmeticOverflow)?;
        let south_north = create_axis_transfers(
            &topology,
            width,
            ExchangeAxis::SouthNorth,
            periodicity.south_north(),
            staggering.south_north(),
        )?;
        let west_east = create_axis_transfers(
            &topology,
            width,
            ExchangeAxis::WestEast,
            periodicity.west_east(),
            staggering.west_east(),
        )?;

        Ok(Self {
            topology,
            halo_width,
            periodicity,
            staggering,
            phases: [south_north, west_east],
        })
    }

    /// Returns the topology used to validate this plan.
    pub const fn topology(&self) -> &DomainTopology {
        &self.topology
    }

    /// Returns the exchanged logical half-width.
    pub const fn halo_width(&self) -> usize {
        self.halo_width
    }

    /// Returns the selected periodic axes.
    pub const fn periodicity(&self) -> HorizontalPeriodicity {
        self.periodicity
    }

    /// Returns the field's horizontal staggering.
    pub const fn staggering(&self) -> HorizontalStaggering {
        self.staggering
    }

    /// Returns the ordered south-north and west-east transfer phases.
    pub fn phases(&self) -> &[Vec<HaloTransfer>; 2] {
        &self.phases
    }

    /// Returns all transfers for one phase.
    pub fn transfers(&self, axis: ExchangeAxis) -> &[HaloTransfer] {
        match axis {
            ExchangeAxis::SouthNorth => &self.phases[0],
            ExchangeAxis::WestEast => &self.phases[1],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BoundaryWidths, DomainBounds, ExchangeDirection, IndexRange, LocalHaloExchange,
        LocalPatchField, ProcessGrid,
    };

    use super::*;

    fn topology(periodic_storage: usize) -> DomainTopology {
        DomainTopology::try_new(
            DomainBounds::new(
                IndexRange::try_new(0, 10).unwrap(),
                IndexRange::try_new(0, 2).unwrap(),
                IndexRange::try_new(0, 8).unwrap(),
            ),
            ProcessGrid::try_new(2, 2).unwrap(),
            2,
            BoundaryWidths::new(periodic_storage, periodic_storage),
        )
        .unwrap()
    }

    fn fill_owned(field: &mut LocalPatchField<i32>) {
        let patches = field.topology().patches().to_vec();
        let bottom_top = field.topology().domain().bottom_top();
        for patch in patches {
            for south_north in
                patch.owned().south_north().start()..patch.owned().south_north().end()
            {
                for level in bottom_top.start()..bottom_top.end() {
                    for west_east in
                        patch.owned().west_east().start()..patch.owned().west_east().end()
                    {
                        let value = west_east + 100 * south_north + 10_000 * level;
                        field
                            .set_value(patch.patch_id(), west_east, level, south_north, value)
                            .unwrap();
                    }
                }
            }
        }
    }

    #[test]
    fn local_exchange_populates_internal_edges_and_corners() {
        let topology = topology(0);
        let plan = HaloExchangePlan::try_new(
            topology.clone(),
            2,
            HorizontalPeriodicity::default(),
            HorizontalStaggering::default(),
        )
        .unwrap();
        let lower_left = topology.patch_id_at(0, 0).unwrap();
        let mut field = LocalPatchField::try_from_value(topology, -1_i32).unwrap();
        fill_owned(&mut field);

        LocalHaloExchange::execute(&plan, &mut field).unwrap();

        assert_eq!(*field.value(lower_left, 5, 0, 4).unwrap(), 405);
        assert_eq!(*field.value(lower_left, 6, 1, 5).unwrap(), 10_506);
    }

    #[test]
    fn periodic_ranges_follow_rsl_lite_endpoint_and_stagger_rules() {
        let topology = topology(3);
        let plan = HaloExchangePlan::try_new(
            topology.clone(),
            2,
            HorizontalPeriodicity::new(true, true),
            HorizontalStaggering::new(true, false),
        )
        .unwrap();
        let lower_left = topology.patch_id_at(0, 0).unwrap();
        let upper_x_transfer = plan
            .transfers(ExchangeAxis::WestEast)
            .iter()
            .find(|transfer| {
                transfer.destination_patch_id() == topology.patch_id_at(1, 0).unwrap()
                    && transfer.direction() == ExchangeDirection::Upper
            })
            .unwrap();
        let lower_x_transfer = plan
            .transfers(ExchangeAxis::WestEast)
            .iter()
            .find(|transfer| {
                transfer.destination_patch_id() == lower_left
                    && transfer.direction() == ExchangeDirection::Lower
            })
            .unwrap();

        assert_eq!(
            lower_x_transfer.source().west_east(),
            IndexRange::try_new(7, 9).unwrap()
        );
        assert_eq!(
            lower_x_transfer.destination().west_east(),
            IndexRange::try_new(-2, 0).unwrap()
        );
        assert_eq!(
            upper_x_transfer.source().west_east(),
            IndexRange::try_new(0, 3).unwrap()
        );
        assert_eq!(
            upper_x_transfer.destination().west_east(),
            IndexRange::try_new(9, 12).unwrap()
        );
    }

    #[test]
    fn invalid_periodic_storage_fails_before_field_mutation() {
        let topology = topology(1);

        let result = HaloExchangePlan::try_new(
            topology,
            2,
            HorizontalPeriodicity::new(true, false),
            HorizontalStaggering::default(),
        );

        assert_eq!(
            result,
            Err(HaloExchangeError::PeriodicBoundaryStorageTooNarrow {
                halo_width: 2,
                boundary_width: 1,
            })
        );
    }
}
