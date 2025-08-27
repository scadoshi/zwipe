use crate::domain::card::models::scryfall_card::card_faces::{CardFace, CardFaces};
use sqlx::{encode::IsNull, types::JsonValue, Decode, Encode, Postgres, Type};

// ===================================
//       impls for individual
// ===================================

impl TryFrom<CardFace> for JsonValue {
    type Error = serde_json::Error;
    fn try_from(value: CardFace) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

impl Decode<'_, Postgres> for CardFace {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let json_value = <JsonValue as Decode<Postgres>>::decode(value)?;
        let related_card: CardFace = serde_json::from_value(json_value)?;
        Ok(related_card)
    }
}

impl Type<Postgres> for CardFace {
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <JsonValue as Type<Postgres>>::compatible(ty)
    }

    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <JsonValue as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for CardFace {
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

// ===================================
//     impls for wrapped vec
// ===================================

impl TryFrom<CardFaces> for JsonValue {
    type Error = serde_json::Error;
    fn try_from(value: CardFaces) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

impl Decode<'_, Postgres> for CardFaces {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let json_value = <JsonValue as Decode<Postgres>>::decode(value)?;
        let related_card: CardFaces = serde_json::from_value(json_value)?;
        Ok(related_card)
    }
}

impl Type<Postgres> for CardFaces {
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <JsonValue as Type<Postgres>>::compatible(ty)
    }

    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <JsonValue as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for CardFaces {
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
