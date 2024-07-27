use axum::routing::method_routing;
use opendal::{Operator, Scheme};
use snafu::prelude::*;
use std::{collections::HashMap, fmt::Debug, io, str::FromStr};
use uuid::Uuid;

use async_graphql::*;
use log::{debug, error, info};

use crate::{
    mount::{FsMounter, Mounter},
    NFSServer,
};

#[derive(Debug, Snafu)]
pub(crate) enum GraphQLError {
    #[snafu(display("Unsupported scheme type {scheme}"))]
    UnsupportedScheme { scheme: String },

    #[snafu(display("NFSServer FS not found in GraphQL context"))]
    NFSServerNotFound,

    #[snafu(display("Fail to create operator with parameters: {source}"))]
    OperatorCreationFailure { source: opendal::Error },

    #[snafu(display("Fail to register operator: {source}"))]
    OperatorRegistrationFailure { source: io::Error },

    #[snafu(display("Fail to mount fs at {mount_point}: {source}"))]
    MountError {
        source: io::Error,
        mount_point: String,
    },
}

#[derive(SimpleObject, Default, Debug)]
pub struct MountedFs {
    pub id: String,
    pub mount_point: String,
    pub scheme: String,
    pub root: String,
    pub name: String,
}

pub struct Query;

trait NFSContext {
    fn nfs_server(&self) -> Result<&NFSServer, GraphQLError>;
}

impl NFSContext for Context<'_> {
    fn nfs_server(&self) -> Result<&NFSServer, GraphQLError> {
        self.data::<NFSServer>()
            .map_err(|_| GraphQLError::NFSServerNotFound)
    }
}

#[Object]
impl Query {
    #[inline]
    async fn fs<'ctx>(&self, ctx: &Context<'ctx>) -> async_graphql::Result<Vec<MountedFs>> {
        let nfs = ctx.nfs_server()?;

        nfs.file_systems()
            .await
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
        mount_point: Option<String>,
    ) -> Result<Uuid, GraphQLError> {
        debug!(
            "mounting {} at {:?} with parameters {:?}",
            service, mount_point, parameters
        );

        let nfs = ctx.nfs_server()?;

        let scheme = Scheme::from_str(&service).map_err(|_| {
            UnsupportedSchemeSnafu {
                scheme: service.clone(),
            }
            .build()
        })?;

        let op = Operator::via_map(scheme, parameters).context(OperatorCreationFailureSnafu {})?;

        let id = nfs
            .register("127.0.0.1:20000", op)
            .await
            .context(OperatorRegistrationFailureSnafu {})?;

        if let Some(mount_point) = mount_point {
            info!("Mounting {} at {}", service, mount_point);

            FsMounter::mount("127.0.0.1", 20000, "", &mount_point, false)
                .await
                .context(MountSnafu {
                    mount_point: mount_point.clone(),
                })?;
        }

        Ok(id)
    }
}
