use std::{collections::HashMap, default, str::FromStr};

use async_graphql::*;
use log::{debug, error};
use opendal::{Operator, Scheme};

use crate::{errors::OpendalMountError, NFSServer};

#[derive(SimpleObject, Default, Debug)]
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
        let nfs = ctx
            .data::<NFSServer>()
            .map_err(|_| OpendalMountError::NFSServerNotFound())?;

        nfs.file_systems()
            .iter()
            .map(|id| {
                Ok(MountedFs {
                    id: id.to_string(),
                    ..Default::default()
                })
            })
            .collect()
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

        let nfs = ctx
            .data::<NFSServer>()
            .map_err(|_| OpendalMountError::NFSServerNotFound())?;

        let scheme = Scheme::from_str(&service)
            .map_err(|_| OpendalMountError::UnsupportedScheme(service))?;

        // let op = Operator::via_map(scheme, parameters).map_err(|e| {
        //     error!("operator creation failure: {}", e);
        //     OpendalMountError::OperatorCreateError(format!("{}", e))
        // })?;

        // mfs.mount_operator(&mount_point, op).await?;

        Ok(mount_point)
    }
}
