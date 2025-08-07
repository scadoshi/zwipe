use std::io::Write;

use diesel::{deserialize::FromSql, pg::Pg, serialize::ToSql, sql_types::VarChar};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl FromSql<VarChar, Pg> for Color {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match String::from_sql(bytes)?.to_lowercase() {
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

impl ToSql for Color {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        match self {
            Self::White => out.write_all(b"W")?,
            Self::Blue => out.write_all(b"U")?,
            Self::Black => out.write_all(b"B")?,
            Self::Red => out.write_all(b"R")?,
            Self::Green => out.write_all(b"G")?,
        }
        Ok(IsNull::No)
    }
}
