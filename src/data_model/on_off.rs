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


use rs_matter::error::{Error, ErrorCode};
use rs_matter::with;

use rs_matter::data_model::objects::{Cluster, Dataver, InvokeContext, ReadContext};

pub use crate::data_model::clusters::on_off::*;

/// A sample implementation of a handler for the On/Off Matter cluster.
#[derive(Clone)]
pub struct OnOffCluster<T> where T: OnOffHooks {
    dataver: Dataver,
    handler: T,
}

impl<T> OnOffCluster<T> where T: OnOffHooks {
    /// Creates a new instance of `OnOffHandler` with the given `Dataver`.
    pub const fn new(dataver: Dataver, handler: T) -> Self {
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
    pub fn set(&self, ctx: &InvokeContext<'_>, on: bool) -> Result<(), Error> {
        if self.handler.raw_get_on_off() != on {
            // todo If there is a LevelControl cluster on the same endpoint, we should
            // set the level to on_level when turning on the light.

            // execute the business logic
            self.handler.set_on(ctx, on)?;

            self.handler.raw_set_on_off(on)?;
            self.dataver.changed();
            ctx.notify_changed();
        }
        Ok(())
    }
}

impl<T> ClusterAsyncHandler for OnOffCluster<T> where T: OnOffHooks {
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
        Ok(self.handler.raw_get_on_off())
    }

    async fn handle_off(&self, ctx: &InvokeContext<'_>) -> Result<(), Error> {
        self.set(ctx, false)
    }

    async fn handle_on(&self, ctx: &InvokeContext<'_>) -> Result<(), Error> {
        self.set(ctx, true)
    }

    async fn handle_toggle(&self, ctx: &InvokeContext<'_>) -> Result<(), Error> {
        self.set(ctx, !self.handler.raw_get_on_off())
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

pub trait OnOffHooks {
    fn raw_get_on_off(&self) -> bool;
    fn raw_set_on_off(&self, on: bool) -> Result<(), Error>;
    fn set_on(&self, ctx: &InvokeContext<'_>, on: bool) -> Result<(), Error>;
}

