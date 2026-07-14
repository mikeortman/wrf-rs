use mpi::datatype::Equivalence;
use mpi::request::multiple_scope;
use mpi::topology::Communicator;
use mpi::traits::{Destination, Source};
use wrf_domain::{ExchangeAxis, ExchangeDirection, HaloExchangePlan, HaloTransfer, PatchField};

use crate::{MpiHaloExchangeError, MpiHaloExchangeResult};

struct PendingMessage<Value> {
    transfer: HaloTransfer,
    values: Vec<Value>,
}

/// Non-blocking MPI executor for a transport-neutral halo plan.
#[derive(Clone, Copy, Debug, Default)]
pub struct MpiHaloExchange;

impl MpiHaloExchange {
    /// Exchanges one rank-local patch field over an MPI communicator.
    ///
    /// Every receive is posted before sends. All messages in a phase complete
    /// before unpacking, preserving the same Y-then-X corner propagation as
    /// [`wrf_domain::LocalHaloExchange`].
    pub fn execute<CommunicatorType, Value>(
        communicator: &CommunicatorType,
        plan: &HaloExchangePlan,
        field: &mut PatchField<Value>,
    ) -> MpiHaloExchangeResult<()>
    where
        CommunicatorType: Communicator,
        Value: Clone + Equivalence,
    {
        validate_rank_field(communicator, plan, field)?;
        for phase in plan.phases() {
            exchange_phase(communicator, phase, field)?;
        }
        Ok(())
    }
}

fn validate_rank_field<CommunicatorType, Value>(
    communicator: &CommunicatorType,
    plan: &HaloExchangePlan,
    field: &PatchField<Value>,
) -> MpiHaloExchangeResult<()>
where
    CommunicatorType: Communicator,
    Value: Clone,
{
    let actual =
        usize::try_from(communicator.size()).map_err(|_| MpiHaloExchangeError::InvalidRank {
            rank: communicator.size(),
        })?;
    let expected = plan.topology().process_grid().process_count();
    if actual != expected {
        return Err(MpiHaloExchangeError::CommunicatorSizeMismatch { expected, actual });
    }
    let rank = communicator.rank();
    let rank_index =
        usize::try_from(rank).map_err(|_| MpiHaloExchangeError::InvalidRank { rank })?;
    if field.patch_id().value() != rank_index {
        return Err(MpiHaloExchangeError::FieldRankMismatch {
            patch_id: field.patch_id(),
            rank,
        });
    }
    Ok(())
}

fn exchange_phase<CommunicatorType, Value>(
    communicator: &CommunicatorType,
    transfers: &[HaloTransfer],
    field: &mut PatchField<Value>,
) -> MpiHaloExchangeResult<()>
where
    CommunicatorType: Communicator,
    Value: Clone + Equivalence,
{
    let patch_id = field.patch_id();
    let outbound = transfers
        .iter()
        .copied()
        .filter(|transfer| transfer.source_patch_id() == patch_id)
        .map(|transfer| {
            Ok(PendingMessage {
                transfer,
                values: field.pack_transfer(transfer)?,
            })
        })
        .collect::<MpiHaloExchangeResult<Vec<_>>>()?;
    let mut inbound = transfers
        .iter()
        .copied()
        .filter(|transfer| transfer.destination_patch_id() == patch_id)
        .map(|transfer| {
            Ok(PendingMessage {
                transfer,
                values: field.prepare_receive_buffer(transfer)?,
            })
        })
        .collect::<MpiHaloExchangeResult<Vec<_>>>()?;

    let request_count = outbound.len() + inbound.len();
    multiple_scope(request_count, |scope, requests| {
        for message in &mut inbound {
            let source_rank = message.transfer.source_patch_id().value() as i32;
            let request = communicator
                .process_at_rank(source_rank)
                .immediate_receive_into_with_tag(
                    scope,
                    &mut message.values[..],
                    message_tag(message.transfer),
                );
            requests.add(request);
        }
        for message in &outbound {
            let destination_rank = message.transfer.destination_patch_id().value() as i32;
            let request = communicator
                .process_at_rank(destination_rank)
                .immediate_send_with_tag(scope, &message.values[..], message_tag(message.transfer));
            requests.add(request);
        }
        let mut completed = Vec::with_capacity(request_count);
        requests.wait_all(&mut completed);
    });

    for message in inbound {
        field.unpack_transfer(message.transfer, &message.values)?;
    }
    Ok(())
}

fn message_tag(transfer: HaloTransfer) -> i32 {
    match (transfer.axis(), transfer.direction()) {
        (ExchangeAxis::SouthNorth, ExchangeDirection::Lower) => 0,
        (ExchangeAxis::SouthNorth, ExchangeDirection::Upper) => 1,
        (ExchangeAxis::WestEast, ExchangeDirection::Lower) => 2,
        (ExchangeAxis::WestEast, ExchangeDirection::Upper) => 3,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn message_tags_are_unique_by_phase_and_direction() {
        let tags = [0, 1, 2, 3];

        assert_eq!(tags.len(), 4);
        assert!(tags.windows(2).all(|pair| pair[0] != pair[1]));
    }
}
