use async_trait::async_trait;

use crate::impls::packet_relayers::base_ack_packet::BaseAckPacketRelayer;
use crate::impls::packet_relayers::base_receive_packet::BaseReceivePacketRelayer;
use crate::impls::packet_relayers::full_relay::FullRelayer;
use crate::impls::packet_relayers::retry::RetryRelayer;
use crate::impls::packet_relayers::skip_received_packet::SkipReceivedPacketRelayer;
use crate::std_prelude::*;
use crate::traits::contexts::ibc_event::HasIbcEvents;
use crate::traits::contexts::relay::RelayContext;
use crate::traits::packet_relayer::PacketRelayer;
use crate::traits::packet_relayers::ack_packet::AckPacketRelayer;
use crate::traits::packet_relayers::receive_packet::ReceivePacketRelayer;
use crate::types::aliases::{Height, WriteAcknowledgementEvent};

pub struct TopRelayer {
    pub relayer: TopRelayer_,
}

pub struct TopReceivePacketRelayer {
    pub relayer: TopReceivePacketRelayer_,
}

pub struct TopAckPacketRelayer {
    pub relayer: BaseAckPacketRelayer,
}

pub type TopRelayer_ = RetryRelayer<FullRelayer<TopReceivePacketRelayer, TopAckPacketRelayer>>;

pub type TopReceivePacketRelayer_ = SkipReceivedPacketRelayer<BaseReceivePacketRelayer>;

impl TopRelayer {
    pub fn new(max_retry: usize) -> Self {
        let relayer1 = FullRelayer {
            receive_relayer: TopReceivePacketRelayer::new(),
            ack_relayer: TopAckPacketRelayer::new(),
        };

        let relayer2 = RetryRelayer::new(max_retry, relayer1);

        TopRelayer { relayer: relayer2 }
    }
}

impl TopReceivePacketRelayer {
    pub fn new() -> Self {
        let relayer = SkipReceivedPacketRelayer::new(BaseReceivePacketRelayer);

        TopReceivePacketRelayer { relayer }
    }
}

impl TopAckPacketRelayer {
    pub fn new() -> Self {
        TopAckPacketRelayer {
            relayer: BaseAckPacketRelayer,
        }
    }
}

#[async_trait]
impl<Relay> PacketRelayer<Relay> for TopRelayer
where
    Relay: RelayContext,
    TopRelayer_: PacketRelayer<Relay>,
{
    async fn relay_packet(
        &self,
        relay: &Relay,
        packet: &Relay::Packet,
    ) -> Result<(), Relay::Error> {
        self.relayer.relay_packet(relay, packet).await
    }
}

#[async_trait]
impl<Relay> ReceivePacketRelayer<Relay> for TopReceivePacketRelayer
where
    Relay: RelayContext,
    Relay::DstChain: HasIbcEvents<Relay::SrcChain>,
    TopReceivePacketRelayer_: ReceivePacketRelayer<Relay>,
{
    async fn relay_receive_packet(
        &self,
        context: &Relay,
        source_height: &Height<Relay::SrcChain>,
        packet: &Relay::Packet,
    ) -> Result<Option<WriteAcknowledgementEvent<Relay::DstChain, Relay::SrcChain>>, Relay::Error>
    {
        self.relayer
            .relay_receive_packet(context, source_height, packet)
            .await
    }
}

#[async_trait]
impl<Relay> AckPacketRelayer<Relay> for TopAckPacketRelayer
where
    Relay: RelayContext,
    Relay::DstChain: HasIbcEvents<Relay::SrcChain>,
    BaseAckPacketRelayer: AckPacketRelayer<Relay>,
{
    async fn relay_ack_packet(
        &self,
        context: &Relay,
        destination_height: &Height<Relay::DstChain>,
        packet: &Relay::Packet,
        ack: &WriteAcknowledgementEvent<Relay::DstChain, Relay::SrcChain>,
    ) -> Result<(), Relay::Error> {
        self.relayer
            .relay_ack_packet(context, destination_height, packet, ack)
            .await
    }
}
