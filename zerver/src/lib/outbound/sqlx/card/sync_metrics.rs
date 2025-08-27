use std::ops::Deref;

use chrono::NaiveDateTime;
use sqlx::{encode::IsNull, types::JsonValue, Decode, Encode, Postgres, Type};
use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::domain::card::models::sync_metrics::{
    ErrorMetrics, SyncMetrics, SyncStatus, SyncType, VecErrorMetrics,
};

// ======================================
//  database compatibility for new types
// ======================================

// ===============
//  error metrics
// ===============

// ======================
//  impls for individual
// ======================

impl TryFrom<ErrorMetrics> for JsonValue {
    type Error = serde_json::Error;
    fn try_from(value: ErrorMetrics) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

impl Decode<'_, Postgres> for ErrorMetrics {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let json_value = <JsonValue as Decode<Postgres>>::decode(value)?;
        let error_metrics: ErrorMetrics = serde_json::from_value(json_value)?;
        Ok(error_metrics)
    }
}

impl Type<Postgres> for ErrorMetrics {
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <JsonValue as Type<Postgres>>::compatible(ty)
    }

    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <JsonValue as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for ErrorMetrics {
    fn encode(
        self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        let json_value: JsonValue = serde_json::to_value(self)?;
        json_value.encode(buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let json_value: JsonValue = serde_json::to_value(self)?;
        json_value.encode(buf)
    }

    fn produces(&self) -> Option<<Postgres as sqlx::Database>::TypeInfo> {
        Some(<JsonValue as Type<Postgres>>::type_info())
    }

    fn size_hint(&self) -> usize {
        0
    }
}

// =======================
//  impls for wrapped vec
// =======================

impl TryFrom<VecErrorMetrics> for JsonValue {
    type Error = serde_json::Error;
    fn try_from(value: VecErrorMetrics) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

impl Decode<'_, Postgres> for VecErrorMetrics {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let json_value = <JsonValue as Decode<Postgres>>::decode(value)?;
        let error_metrics_vec: VecErrorMetrics = serde_json::from_value(json_value)?;
        Ok(error_metrics_vec)
    }
}

impl Type<Postgres> for VecErrorMetrics {
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <JsonValue as Type<Postgres>>::compatible(ty)
    }

    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <JsonValue as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for VecErrorMetrics {
    fn encode(
        self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        let json_value: JsonValue = serde_json::to_value(self)?;
        json_value.encode(buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let json_value: JsonValue = serde_json::to_value(self)?;
        json_value.encode(buf)
    }

    fn produces(&self) -> Option<<Postgres as sqlx::Database>::TypeInfo> {
        Some(<JsonValue as Type<Postgres>>::type_info())
    }

    fn size_hint(&self) -> usize {
        0
    }
}

// ==============
//  sync metrics
// ==============

#[derive(Debug, FromRow)]
pub struct DatabaseSyncMetrics {
    #[sqlx(rename = "id")]
    _id: Uuid,
    sync_type: String,
    started_at: NaiveDateTime,
    ended_at: Option<NaiveDateTime>,
    duration_in_seconds: i32,
    status: String,
    received: i32,
    imported: i32,
    skipped: i32,
    error_count: i32,
    errors: VecErrorMetrics,
}

impl TryFrom<DatabaseSyncMetrics> for SyncMetrics {
    type Error = anyhow::Error;
    fn try_from(value: DatabaseSyncMetrics) -> anyhow::Result<Self> {
        let sync_type = SyncType::try_from(value.sync_type.as_str())?;
        let status = SyncStatus::try_from(value.status.as_str())?;
        let errors: Vec<ErrorMetrics> = value.errors.deref().to_vec();

        Ok(Self::new(
            sync_type,
            value.started_at,
            value.ended_at,
            value.duration_in_seconds,
            status,
            value.received,
            value.imported,
            value.skipped,
            value.error_count,
            errors,
        ))
    }
}
