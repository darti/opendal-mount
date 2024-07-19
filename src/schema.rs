use opendal::{EntryMode, Operator, Scheme};
use snafu::prelude::*;
use std::{collections::HashMap, fmt::Debug, str::FromStr};

use async_graphql::*;
use log::{debug, error};

use crate::NFSServer;

#[derive(Debug, Snafu)]
pub(crate) enum GraphQLError {
    #[snafu(display("Unsupported scheme type {scheme}"))]
    UnsupportedScheme { scheme: String },

    #[snafu(display("NFSServer FS not found in GraphQL context"))]
    NFSServerNotFound,

    #[snafu(display("Fail to create operator with parameters: {source}"))]
    OperatorCreationFailure { source: opendal::Error },
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
    ) -> Result<String, GraphQLError> {
        debug!(
            "mounting {} at {} with parameters {:?}",
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

        // let op = Operator::via_map(scheme, parameters).map_err(|e| {
        //     error!("operator creation failure: {}", e);
        //     OpendalMountError::OperatorCreateError(format!("{}", e))
        // })?;

        // mfs.mount_operator(&mount_point, op).await?;

        Ok(mount_point)
    }
}
