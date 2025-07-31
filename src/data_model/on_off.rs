/*
 *
 *    Copyright (c) 2020-2022 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! This module contains the implementation of the On/Off cluster and its handler.
//!
//! While this cluster is not necessary for the operation of `rs-matter`, this
//! implementation is useful in examples and tests.

use core::cell::Cell;

use rs_matter::error::{Error, ErrorCode};
use rs_matter::with;

use rs_matter::data_model::objects::{Cluster, Dataver, InvokeContext, ReadContext};
use crate::led::led::{LedSender, ControlMessage};

pub use crate::data_model::clusters::on_off::*;

/// A sample implementation of a handler for the On/Off Matter cluster.
#[derive(Clone)]
pub struct OnOffHandler<'a> {
    dataver: Dataver,
    on: Cell<bool>,
    sender: LedSender<'a>,
}

impl<'a> OnOffHandler<'a> {
    /// Creates a new instance of `OnOffHandler` with the given `Dataver`.
    pub const fn new(dataver: Dataver, sender: LedSender<'a>) -> Self {
        Self {
            dataver,
            on: Cell::new(false),
            sender,
        }
    }

    /// Adapt the handler instance to the generic `rs-matter` `Handler` trait
    pub const fn adapt(self) -> HandlerAsyncAdaptor<Self> {
        HandlerAsyncAdaptor(self)
    }

    /// Return the current state of the On/Off attribute.
    pub fn get(&self) -> bool {
        self.on.get()
    }

    /// Set the On/Off attribute to the given value and notify potential subscribers.
    pub async fn set(&self, on: bool) {
        if self.on.get() != on {
            if on {
                // todo: get on_level from levelControl
                self.sender.send(ControlMessage::SetOn(Some(150))).await;
            } else {
                self.sender.send(ControlMessage::SetOn(None)).await;
            }
            self.on.set(on);
            self.dataver.changed();
        }
    }
}

impl<'a> ClusterAsyncHandler for OnOffHandler<'a> {
    const CLUSTER: Cluster<'static> = FULL_CLUSTER
        .with_revision(1)
        .with_attrs(with!(required))
        .with_cmds(with!(CommandId::On | CommandId::Off | CommandId::Toggle));

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }

    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    async fn on_off(&self, _ctx: &ReadContext<'_>) -> Result<bool, Error> {
        Ok(self.on.get())
    }

    async fn handle_off(&self, ctx: &InvokeContext<'_>) -> Result<(), Error> {
        self.set(false).await;
        ctx.notify_changed();
        Ok(())
    }

    async fn handle_on(&self, ctx: &InvokeContext<'_>) -> Result<(), Error> {
        self.set(true).await;
        ctx.notify_changed();
        Ok(())
    }

    async fn handle_toggle(&self, ctx: &InvokeContext<'_>) -> Result<(), Error> {
        self.set(!self.on.get()).await;
        ctx.notify_changed();
        Ok(())
    }

    async fn handle_off_with_effect(
        &self,
        _ctx: &InvokeContext<'_>,
        _request: OffWithEffectRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    async fn handle_on_with_recall_global_scene(&self, _ctx: &InvokeContext<'_>) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    async fn handle_on_with_timed_off(
        &self,
        _ctx: &InvokeContext<'_>,
        _request: OnWithTimedOffRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }
}
