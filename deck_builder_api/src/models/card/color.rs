use derive_more::{Deref, DerefMut, Display, From, IntoIterator};
use diesel::{
    deserialize::FromSql,
    pg::Pg,
    serialize::{IsNull, ToSql},
    sql_types::{Array, Nullable, Text},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
type FancyError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
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

impl FromSql<Text, Pg> for Color {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let bytes_str = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        match bytes_str.as_str() {
            "white" | "w" => Ok(Self::White),
            "blue" | "u" => Ok(Self::Blue),
            "black" | "b" => Ok(Self::Black),
            "red" | "r" => Ok(Self::Red),
            "green" | "g" => Ok(Self::Green),
            x => {
                return Err(
                    format!("Failed to convert given value: {:?} into a valid Color", x).into(),
                )
            }
        }
    }
}

impl ToSql<Text, Pg> for Color {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        write!(out, "{}", self)?;
        Ok(IsNull::No)
    }
}

impl TryFrom<&str> for Color {
    type Error = FancyError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "white" | "w" => Ok(Self::White),
            "blue" | "u" => Ok(Self::Blue),
            "black" | "b" => Ok(Self::Black),
            "red" | "r" => Ok(Self::Red),
            "green" | "g" => Ok(Self::Green),
            x => {
                return Err(
                    format!("Failed to convert given value: {:?} into a valid Color", x).into(),
                )
            }
        }
    }
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Deref,
    DerefMut,
    IntoIterator,
    From,
    AsExpression,
    FromSqlRow,
)]
#[diesel(sql_type = Array<Nullable<Text>>)]
pub struct Colors(Vec<Option<Color>>);
impl FromSql<Array<Nullable<Text>>, Pg> for Colors {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let strings = <Vec<Option<String>> as FromSql<Array<Nullable<Text>>, Pg>>::from_sql(bytes)?;
        let colors: Result<Vec<Option<Color>>, FancyError> = strings
            .into_iter()
            .map(|opt_s| opt_s.map(|s| Color::try_from(s.as_str())).transpose())
            .collect();

        Ok(Colors(colors?))
    }
}

impl ToSql<Array<Nullable<Text>>, Pg> for Colors {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        write!(out, "{{")?;
        for (i, opt_color) in self.iter().enumerate() {
            if i > 0 {
                write!(out, ",")?;
            }
            match opt_color {
                Some(color) => write!(out, "\"{}\"", color)?,
                None => write!(out, "NULL")?,
            }
        }
        write!(out, "}}")?;
        Ok(IsNull::No)
    }
}
