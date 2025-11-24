use serde::{Deserialize, Serialize};

/// some updates were represented by Option<Option<T>>
/// which was problematic because `Some(None)` serializes to json `null`
/// then deserializes to just `None` so is lossy
///
/// this wrapper prevents that
///
/// optional + update => opt + date => optdate
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Optdate<T> {
    Set(Option<T>),
    Unchanged,
}

impl<T> Optdate<T>
where
    T: PartialEq,
{
    pub fn is_unchanged(&self) -> bool {
        *self == Optdate::Unchanged
    }

    pub fn is_changed(&self) -> bool {
        !self.is_unchanged()
    }

    pub fn into_option(self) -> Option<Option<T>> {
        self.into()
    }
}

impl<T> Iterator for Optdate<T> {
    type Item = Option<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Optdate::Set(value) => {
                let result = Some(value.take());
                *self = Optdate::Unchanged;
                result
            }
            Optdate::Unchanged => None,
        }
    }
}

impl<T> From<Optdate<T>> for Option<Option<T>>
where
    T: PartialEq,
{
    fn from(value: Optdate<T>) -> Self {
        match value {
            Optdate::Set(inner) => Some(inner),
            Optdate::Unchanged => None,
        }
    }
}

// skipping this as Option<T> will work for now
// but maybe use later

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Reqdate<T> {
//     Set(T),
//     None,
// }
