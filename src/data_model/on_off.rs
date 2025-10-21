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

use log::info;
use rs_matter_embassy::matter::error::{Error, ErrorCode};
use rs_matter_embassy::matter::with;

use rs_matter_embassy::matter::dm::{Cluster, Dataver, InvokeContext, ReadContext};

pub use crate::data_model::clusters::on_off::*;

/// A sample implementation of a handler for the On/Off Matter cluster.
#[derive(Clone)]
pub struct OnOffCluster<'a, T: OnOffHooks>  {
    dataver: Dataver,
    handler: &'a T,
}

impl<'a, T: OnOffHooks> OnOffCluster<'a, T> {
    /// Creates a new instance of `OnOffHandler` with the given `Dataver`.
    pub const fn new(dataver: Dataver, handler: &'a T) -> Self {
        Self {
            dataver,
            handler,
        }
    }

    /// Adapt the handler instance to the generic `rs-matter` `Handler` trait
    pub const fn adapt(self) -> HandlerAsyncAdaptor<Self> {
        HandlerAsyncAdaptor(self)
    }

    /// Set the On/Off attribute to the given value and notify potential subscribers.
    pub fn set(&self, ctx: impl InvokeContext, on: bool) -> Result<(), Error> {
        if self.handler.raw_get_on_off() != on {
            // todo If there is a LevelControl cluster on the same endpoint, we should
            // set the level to on_level when turning on the light.

            // execute the business logic
            self.handler.set_on(&ctx, on)?;

            self.handler.raw_set_on_off(on)?;
            self.dataver.changed();
            ctx.notify_changed();
        }
        Ok(())
    }
}

impl<'a, T: OnOffHooks> ClusterAsyncHandler for OnOffCluster<'a, T> {
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

    async fn on_off(&self, _ctx: impl ReadContext) -> Result<bool, Error> {
        info!("OnOff: Called on_off()");
        Ok(self.handler.raw_get_on_off())
    }

    async fn handle_off(&self, ctx: impl InvokeContext) -> Result<(), Error> {
        info!("OnOff: Called handle_off()");
        self.set(ctx, false)
    }

    async fn handle_on(&self, ctx: impl InvokeContext) -> Result<(), Error> {
        info!("OnOff: Called handle_on()");
        self.set(ctx, true)
    }

    async fn handle_toggle(&self, ctx: impl InvokeContext) -> Result<(), Error> {
        info!("OnOff: Called handle_toggle()");
        self.set(ctx, !self.handler.raw_get_on_off())
    }

    async fn handle_off_with_effect(
        &self,
        _ctx: impl InvokeContext,
        _request: OffWithEffectRequest<'_>,
    ) -> Result<(), Error> {
        info!("OnOff: Called handle_off_with_effect()");
        Err(ErrorCode::InvalidCommand.into())
    }

    async fn handle_on_with_recall_global_scene(&self, _ctx: impl InvokeContext) -> Result<(), Error> {
        info!("OnOff: Called handle_on_with_recall_global_scene()");
        Err(ErrorCode::InvalidCommand.into())
    }

    async fn handle_on_with_timed_off(
        &self,
        _ctx: impl InvokeContext,
        _request: OnWithTimedOffRequest<'_>,
    ) -> Result<(), Error> {
        info!("OnOff: Called handle_on_with_timed_off()");
        Err(ErrorCode::InvalidCommand.into())
    }
}

pub trait OnOffHooks {
    fn raw_get_on_off(&self) -> bool;
    fn raw_set_on_off(&self, on: bool) -> Result<(), Error>;
    fn set_on(&self, ctx: impl InvokeContext, on: bool) -> Result<(), Error>;
}

