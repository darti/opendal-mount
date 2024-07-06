use std::{collections::HashMap, str::FromStr};

use async_graphql::*;
use log::{debug, error};
use opendal::{Operator, Scheme};

use crate::{errors::OpendalMountError, MultiplexedFs};

#[derive(SimpleObject)]
pub struct MountedFs {
    pub id: String,
    pub mount_point: String,
    pub scheme: String,
    pub root: String,
    pub name: String,
}

pub struct Query;

#[Object]
impl Query {
    async fn fs<'ctx>(&self, ctx: &Context<'ctx>) -> async_graphql::Result<Vec<MountedFs>> {
        let mfs = ctx
            .data::<MultiplexedFs>()
            .map_err(|_| OpendalMountError::MultiplexedNotFound())?;

        Ok(mfs.mounted_operators().await)
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn mount<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        service: String,
        parameters: HashMap<String, String>,
        mount_point: String,
    ) -> async_graphql::Result<String> {
        debug!("mounting {} at {}", service, mount_point);

        let mfs = ctx.data::<MultiplexedFs>().map_err(|e| {
            error!("Multiplexed FS not found: {:#?}", e);
            OpendalMountError::MultiplexedNotFound()
        })?;

        let scheme = Scheme::from_str(&service)
            .map_err(|_| OpendalMountError::UnsupportedScheme(service))?;

        let op = Operator::via_map(scheme, parameters).map_err(|e| {
            error!("operator creation failure: {}", e);
            OpendalMountError::OperatorCreateError(format!("{}", e))
        })?;

        mfs.mount_operator(&mount_point, op).await?;

        Ok(mount_point)
    }
}
