use crate::domain::card::models::scryfall_data::language::Language;
use sqlx::{encode::IsNull, postgres::PgTypeInfo, Decode, Encode, Postgres, Type, TypeInfo};

impl Type<Postgres> for Language {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        let name = ty.name().to_lowercase();
        name == "text"
    }
}

impl Encode<'_, Postgres> for Language {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&str as Encode<'_, Postgres>>::encode(self.to_code(), buf)
    }
}

impl Decode<'_, Postgres> for Language {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let string: String = <String as Decode<'_, Postgres>>::decode(value)?;
        Language::try_from(string).map_err(|e| e.into())
    }
}
