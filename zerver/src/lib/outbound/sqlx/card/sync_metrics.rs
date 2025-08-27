use sqlx::{encode::IsNull, types::JsonValue, Decode, Encode, Postgres, Type};

use crate::domain::card::models::sync_metrics::{ErrorMetrics, ErrorMetricsVec};

// ====================================================
//    database compatibility for new types
// ====================================================

// ===========================
//        error metrics
// ===========================

// ===================================
//       impls for individual
// ===================================

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

// ===================================
//     impls for wrapped vec
// ===================================

impl TryFrom<ErrorMetricsVec> for JsonValue {
    type Error = serde_json::Error;
    fn try_from(value: ErrorMetricsVec) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

impl Decode<'_, Postgres> for ErrorMetricsVec {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let json_value = <JsonValue as Decode<Postgres>>::decode(value)?;
        let error_metrics_vec: ErrorMetricsVec = serde_json::from_value(json_value)?;
        Ok(error_metrics_vec)
    }
}

impl Type<Postgres> for ErrorMetricsVec {
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <JsonValue as Type<Postgres>>::compatible(ty)
    }

    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <JsonValue as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for ErrorMetricsVec {
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
