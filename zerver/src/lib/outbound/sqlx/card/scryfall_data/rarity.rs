use crate::domain::card::models::scryfall_data::rarity::{Rarities, Rarity};
use sqlx::{encode::IsNull, postgres::PgTypeInfo, Decode, Encode, Postgres, Type, TypeInfo};

impl Type<Postgres> for Rarity {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        let name = ty.name().to_lowercase();
        name == "text"
    }
}

impl Encode<'_, Postgres> for Rarity {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <String as Encode<'_, Postgres>>::encode(self.to_short_name(), buf)
    }
}

impl Decode<'_, Postgres> for Rarity {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let string: String = <String as Decode<'_, Postgres>>::decode(value)?;
        Rarity::try_from(string).map_err(|e| e.into())
    }
}

impl Type<Postgres> for Rarities {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        PgTypeInfo::with_name("_text")
    }

    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        let name = ty.name().to_lowercase();
        name == "_text" || name == "text[]"
    }
}

impl Encode<'_, Postgres> for Rarities {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        let short_names: Vec<String> = self.to_short_names();
        short_names.encode(buf)
    }
}
