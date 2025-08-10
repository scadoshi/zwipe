use derive_more::Display;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Postgres, Type};

type FancyError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Display)]
pub enum Color {
    #[display("W")]
    White,
    #[display("U")]
    Blue,
    #[display("B")]
    Black,
    #[display("R")]
    Red,
    #[display("G")]
    Green,
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<&str> for Color {
    type Error = FancyError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "w" | "white" => Ok(Self::White),
            "u" | "blue" => Ok(Self::Blue),
            "b" | "black" => Ok(Self::Black),
            "r" | "red" => Ok(Self::Red),
            "g" | "green" => Ok(Self::Green),
            s => Err(format!("Invalid color: {}", s).into()),
        }
    }
}

impl Type<Postgres> for Color {
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        ty.to_string().contains("text")
    }
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <String as Type<Postgres>>::type_info()
    }
}

impl<'r> Decode<'r, Postgres> for Color {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Self::try_from(value.as_str()?)
    }
}

impl<'q> Encode<'q, Postgres> for Color {
    fn encode(
        self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        <String as Encode<'q, Postgres>>::encode(self.to_string(), buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as Encode<'q, Postgres>>::encode_by_ref(&self.to_string(), buf)
    }

    fn produces(&self) -> Option<<Postgres as sqlx::Database>::TypeInfo> {
        Some(Self::type_info())
    }

    fn size_hint(&self) -> usize {
        1
    }
}
