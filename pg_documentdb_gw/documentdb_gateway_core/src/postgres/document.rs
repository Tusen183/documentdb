/*-------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation.  All rights reserved.
 *
 * documentdb_gateway_core/src/postgres/document.rs
 *
 *-------------------------------------------------------------------------
 */

use bson::RawDocument;
use tokio_postgres::types::private::BytesMut;
use tokio_postgres::types::{to_sql_checked, FromSql, ToSql, Type};

/// Provides ability to bind Raw bson in and out of postgres
pub struct PgDocument<'a>(pub &'a RawDocument);

// Custom Debug implementation to show readable JSON instead of hex
impl std::fmt::Debug for PgDocument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Try to parse and display as JSON, fallback to raw format
        match bson::from_slice::<bson::Document>(self.0.as_bytes()) {
            Ok(doc) => write!(f, "PgDocument({})", doc),
            Err(_) => write!(f, "PgDocument({:?})", self.0),
        }
    }
}

// To support multi coordinator scenarios, bind the raw bson into a bytea rather than bson
impl ToSql for PgDocument<'_> {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        self.0.as_bytes().to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool
    where
        Self: Sized,
    {
        ty == &Type::BYTEA
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for PgDocument<'a> {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(PgDocument(RawDocument::from_bytes(raw)?))
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "bson"
    }
}
