use std::{collections::VecDeque, time::Instant};

use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    commands::Command,
    types::ProtocolContext,
    wire::{
        deser::{Deserialize, Deserializer},
        packet::{ControlBody, InnerBody, PacketBody, ReliableBody},
    },
};

use super::{ReliableReceiver, ReliableSender, SplitReceiver, SplitSender};

pub(crate) struct Channel {
    unreliable_out: VecDeque<InnerBody>,

    reliable_in: ReliableReceiver,
    reliable_out: ReliableSender,

    split_in: SplitReceiver,
    split_out: SplitSender,

    to_controller: UnboundedSender<Result<Command>>,
    now: Instant,
    recv_context: ProtocolContext,
    send_context: ProtocolContext,
}

impl Channel {
    pub(crate) fn new(
        remote_is_server: bool,
        to_controller: UnboundedSender<Result<Command>>,
    ) -> Self {
        Self {
            unreliable_out: VecDeque::new(),
            reliable_in: ReliableReceiver::new(),
            reliable_out: ReliableSender::new(),
            split_in: SplitReceiver::new(),
            split_out: SplitSender::new(),
            to_controller,
            now: Instant::now(),
            recv_context: ProtocolContext::latest_for_receive(remote_is_server),
            send_context: ProtocolContext::latest_for_send(remote_is_server),
        }
    }

    pub(crate) fn update_now(&mut self, now: &Instant) {
        self.now = *now;
    }

    pub(crate) fn update_context(
        &mut self,
        recv_context: ProtocolContext,
        send_context: ProtocolContext,
    ) {
        self.recv_context = recv_context;
        self.send_context = send_context;
    }

    /// Process a packet received from remote
    /// Possibly dispatching one or more Commands
    pub(crate) fn process(&mut self, body: PacketBody) -> Result<()> {
        match body {
            PacketBody::Reliable(rb) => self.process_reliable(rb)?,
            PacketBody::Inner(ib) => self.process_inner(ib)?,
        }
        Ok(())
    }

    pub(crate) fn process_reliable(&mut self, body: ReliableBody) -> Result<()> {
        self.reliable_in.push(body);
        while let Some(inner) = self.reliable_in.pop() {
            self.process_inner(inner)?;
        }
        Ok(())
    }

    pub(crate) fn process_inner(&mut self, body: InnerBody) -> Result<()> {
        match body {
            InnerBody::Control(body) => self.process_control(body),
            InnerBody::Original(body) => {
                if let Some(command) = body.command {
                    self.process_command(command);
                }
            }
            InnerBody::Split(body) => {
                if let Some(payload) = self.split_in.push(self.now, body)? {
                    let mut buf = Deserializer::new(self.recv_context, &payload);
                    if let Some(command) = Command::deserialize(&mut buf)? {
                        self.process_command(command);
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn process_control(&mut self, body: ControlBody) {
        if let ControlBody::Ack(ack) = body {
            self.reliable_out.process_ack(&ack);
        } else {
            // Everything else is handled one level up
        }
    }

    pub(crate) fn process_command(&mut self, command: Command) {
        match self.to_controller.send(Ok(command)) {
            Ok(()) => (),
            Err(error) => panic!("Unexpected command channel shutdown: {error:?}"),
        }
    }

    /// Send command to remote
    pub(crate) fn send(&mut self, reliable: bool, command: Command) -> Result<()> {
        let bodies = self.split_out.push(self.send_context, command)?;
        for body in bodies {
            self.send_inner(reliable, body);
        }
        Ok(())
    }

    pub(crate) fn send_inner(&mut self, reliable: bool, body: InnerBody) {
        if reliable {
            self.reliable_out.push(body);
        } else {
            self.unreliable_out.push_back(body);
        }
    }

    /// Check if the channel has anything ready to send.
    pub(crate) fn next_send(&mut self, now: Instant) -> Option<PacketBody> {
        if let Some(body) = self.unreliable_out.pop_front() {
            return Some(PacketBody::Inner(body));
        }
        if let Some(body) = self.reliable_out.pop(now) {
            return Some(body);
        }
        None
    }

    /// Only call after exhausting `next_send()`
    pub(crate) fn next_timeout(&mut self) -> Option<Instant> {
        self.reliable_out.next_timeout()
    }
}
