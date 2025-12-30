use crate::domain::card::models::scryfall_data::colors::{Color, Colors};
use sqlx::{encode::IsNull, postgres::PgTypeInfo, Decode, Encode, Postgres, Type, TypeInfo};

// Colors maps to PostgreSQL TEXT[]
// Encodes as Vec<String> of short names (["W", "U", "B"])
// Decodes from Vec<String> back to Colors

impl Type<Postgres> for Colors {
    fn type_info() -> PgTypeInfo {
        // _text is PostgreSQL's internal name for text[]
        PgTypeInfo::with_name("_text")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        let name = ty.name().to_lowercase();
        name == "_text" || name == "text[]"
    }
}

impl Encode<'_, Postgres> for Colors {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        let short_names: Vec<String> = self.to_short_names();
        short_names.encode(buf)
    }
}

impl Decode<'_, Postgres> for Colors {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let strings: Vec<String> = Vec::<String>::decode(value)?;
        let colors: Result<Vec<Color>, _> = strings
            .iter()
            .map(|s| Color::try_from(s.as_str()))
            .collect();
        colors
            .map(Colors::from)
            .map_err(|_| "invalid color value in database".into())
    }
}
