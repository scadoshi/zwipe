use thiserror::Error;

#[derive(Debug, Error)]
#[error("must be greater than 0")]
pub struct InvalidQuantity;

#[derive(Debug, Error)]
#[error("must be less or greater than 0")]
pub struct InvalidUpdateQuanity;

#[derive(Debug, Clone)]
pub struct Quantity(i32);

impl Quantity {
    pub fn new(quantity: i32) -> Result<Self, InvalidQuantity> {
        if quantity < 1 {
            return Err(InvalidQuantity);
        }
        Ok(Self(quantity))
    }

    pub fn quantity(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct UpdateQuantity(i32);

impl UpdateQuantity {
    pub fn new(add_quantity: i32) -> Result<Self, InvalidUpdateQuanity> {
        if add_quantity == 0 {
            return Err(InvalidUpdateQuanity);
        }
        Ok(Self(add_quantity))
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}
