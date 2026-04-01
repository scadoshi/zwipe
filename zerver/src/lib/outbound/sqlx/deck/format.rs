//! Maps `Format` to PostgreSQL `TEXT` using the legality key (snake_case).

use crate::domain::deck::models::deck::format::Format;
use sqlx::{encode::IsNull, postgres::PgTypeInfo, Decode, Encode, Postgres, Type, TypeInfo};

impl Type<Postgres> for Format {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        let name = ty.name().to_lowercase();
        name == "text" || name == "varchar"
    }
}

impl Encode<'_, Postgres> for Format {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&str as Encode<'_, Postgres>>::encode(self.to_legality_key(), buf)
    }
}

impl Decode<'_, Postgres> for Format {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let string: String = <String as Decode<'_, Postgres>>::decode(value)?;
        Format::try_from(string).map_err(|e| e.into())
    }
}
