use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    dsl::IsNull,
    pg::Pg,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::{Text, VarChar},
};
use serde::{Deserialize, Serialize};

/// An object describing the legality of this card across play formats.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Legalities {
    pub standard: Option<LegalityKind>,
    pub future: Option<LegalityKind>,
    pub historic: Option<LegalityKind>,
    pub timeless: Option<LegalityKind>,
    pub gladiator: Option<LegalityKind>,
    pub pioneer: Option<LegalityKind>,
    pub modern: Option<LegalityKind>,
    pub legacy: Option<LegalityKind>,
    pub pauper: Option<LegalityKind>,
    pub vintage: Option<LegalityKind>,
    pub penny: Option<LegalityKind>,
    pub commander: Option<LegalityKind>,
    pub oathbreaker: Option<LegalityKind>,
    pub standardbrawl: Option<LegalityKind>,
    pub brawl: Option<LegalityKind>,
    pub alchemy: Option<LegalityKind>,
    pub paupercommander: Option<LegalityKind>,
    pub duel: Option<LegalityKind>,
    pub oldschool: Option<LegalityKind>,
    pub premodern: Option<LegalityKind>,
    pub predh: Option<LegalityKind>,
    pub explorer: Option<LegalityKind>,
    pub historicbrawl: Option<LegalityKind>,
}

/// Possible legality states for a format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LegalityKind {
    #[default]
    Legal,
    NotLegal,
    Restricted,
    Banned,
}

impl FromSql<VarChar, Pg> for LegalityKind {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.to_lowercase() {
            "legal" => Ok(LegalityKind::Legal),
            "not_legal" => Ok(LegalityKind::NotLegal),
            "restricted" => Ok(LegalityKind::Restricted),
            "banned" => Ok(LegalityKind::Banned),
            x => Err(format!(
                "Failed to conver given value: {:?} to a valid LegalityKind",
                x
            )
            .into()),
        }
    }
}

impl ToSql<VarChar, Pg> for LegalityKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            LegalityKind::Legal => out.write_all(b"legal")?,
            LegalityKind::NotLegal => out.write_all(b"not_legal")?,
            LegalityKind::Restricted => out.write_all(b"restricted")?,
            LegalityKind::Banned => out.write_all(b"banned")?,
        }
        Ok(IsNull::No)
    }
}
