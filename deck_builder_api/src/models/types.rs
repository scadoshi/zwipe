use diesel::pg::Pg;
use diesel::sql_types::Text;
use diesel::{
    deserialize::{self, FromSql},
    serialize::{self, ToSql},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use strum::{Display, EnumString};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, AsExpression, FromSqlRow, Display, EnumString,
)]
#[diesel(sql_type = Text)]
#[strum(serialize_all = "PascalCase")]
pub enum MtgFormat {
    Standard,
    Modern,
    Pioneer,
    Legacy,
    Vintage,
    Commander,
    Pauper,
    Draft,
    Sealed,
    Brawl,
}

impl ToSql<Text, Pg> for MtgFormat {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.to_string().as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<Text, Pg> for MtgFormat {
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> deserialize::Result<Self> {
        let s = std::str::from_utf8(bytes.as_bytes())?;
        s.parse()
            .map_err(|e| format!("Invalid MtgFormat: {}", e).into())
    }
}
