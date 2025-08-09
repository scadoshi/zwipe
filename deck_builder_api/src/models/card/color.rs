use derive_more::{Deref, DerefMut, Display, From, IntoIterator};
use itertools::Itertools;
use std::io::Write;

use diesel::{
    deserialize::FromSql,
    pg::Pg,
    serialize::{IsNull, ToSql},
    sql_types::{Array, Nullable, Text},
};
use serde::{Deserialize, Serialize};

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

// impl ToSql<Text, Pg> for Color {
//     fn to_sql<'b>(
//         &'b self,
//         out: &mut diesel::serialize::Output<'b, '_, Pg>,
//     ) -> diesel::serialize::Result {
//         match self {
//             Self::White => out.write_all(b"W")?,
//             Self::Blue => out.write_all(b"U")?,
//             Self::Black => out.write_all(b"B")?,
//             Self::Red => out.write_all(b"R")?,
//             Self::Green => out.write_all(b"G")?,
//         }
//         Ok(IsNull::No)
//     }
// }

impl TryFrom<&str> for Color {
    type Error = Box<dyn std::error::Error + Send + Sync>;
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

#[derive(Debug, Clone, Serialize, Deserialize, Deref, DerefMut, IntoIterator, From)]
pub struct Colors(Vec<Color>);

impl FromSql<Text, Pg> for Colors {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let bytes_str: String = <String as FromSql<Text, Pg>>::from_sql(bytes)?.to_lowercase();
        Ok(Colors(
            bytes_str
                .replace("{", "")
                .replace("}", "")
                .split(",")
                .filter(|x| *x != "")
                .map(|x| Color::try_from(x.replace("'", "").as_str()))
                .collect::<Result<Vec<Color>, Box<dyn std::error::Error + Send + Sync>>>()?,
        ))
    }
}

impl ToSql<Array<Nullable<Text>>, Pg> for Colors {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        let adj = self
            .0
            .iter()
            .map(|x| "'".to_string() + x.to_string().as_ref() + "'")
            .join(",");
        let output = "{".to_string() + adj.as_ref() + "}";
        out.write_all(output.as_bytes())?;
        Ok(IsNull::No)
    }
}
