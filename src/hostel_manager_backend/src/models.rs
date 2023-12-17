use std::borrow;
use candid::{Encode, Decode};

use crate::error;

#[derive(candid::CandidType, candid::Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Occupant(pub String);
impl Occupant {
    pub fn new(id: candid::Principal) -> Self {
        Occupant(id.to_string())
    }
}

// #[derive(candid::CandidType, candid::Deserialize, Clone)]
// pub struct CustomVec<T: ic_stable_structures::BoundedStorable>(pub Vec<T>);

#[derive(candid::CandidType, candid::Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum RoomState {
    Full,
    PartiallyOccupied,
    TotallyVacant,
}

#[derive(candid::CandidType, candid::Deserialize, Clone, Debug)]
pub struct Room {
    pub no: u64,
    pub capacity: u64,
    pub price_per_occupant: u64,
    pub state: RoomState,
    pub occupants: Vec<Occupant>,
    pub owner: Occupant,
}

impl Room {
    pub fn new(number: u64, capacity: u64, price_per_occupant: u64, owner: Occupant) -> Self {
        Room {
            no: number,
            capacity,
            price_per_occupant,
            state: RoomState::TotallyVacant,
            occupants: vec![],
           owner,
        }
    }

    pub fn check_price(&self, price: u64) -> bool {
        return price == self.price_per_occupant;
    }

    pub fn is_full(&self) -> bool {
        self.state == RoomState::Full
    }

    pub fn is_owner(&self, owner: Occupant) -> bool {
        self.owner == owner
    }

    pub fn has_occupant(&self, occupant: Occupant) -> Option<usize> {
        self.occupants.iter().position(|o| o == &occupant)
    }

    pub fn add_occupant(&mut self, occupant: Occupant) -> Result<(), error::Error> {
        if self.is_full() {
            return Err(error::Error::RoomFull);
        }

        // ic_cdk::println!("occupant: {:?}", occupant);
        // ic_cdk::println!("occupants: {:?}", self.occupants);
        // ic_cdk::println!("room(self): {:?}", self);
        
        match self.has_occupant(occupant.clone()) {
            Some(_) => Err(error::Error::RoomAlreadyBooked),
            None => {
                self.occupants.push(occupant);
                self.state = if self.occupants.len() == self.capacity as usize {
                    RoomState::Full
                } else {
                    RoomState::PartiallyOccupied
                };
                // ic_cdk::println!("occupants: {:?}", self.occupants);
                Ok(())
            }
        }
    }

    pub fn remove_occupant(&mut self, occupant: Occupant) -> Result<(), error::Error> {
        match self.has_occupant(occupant) {
            Some(index) => {
                self.occupants.remove(index);
                self.state = if self.occupants.len() == 0 {
                    RoomState::TotallyVacant
                } else {
                    RoomState::PartiallyOccupied
                };
                Ok(())
            }
            None => Err(error::Error::NotInRoom),
        }
    }
}

impl ic_stable_structures::Storable for Occupant {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::BoundedStorable for Occupant {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl ic_stable_structures::Storable for Room {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::BoundedStorable for Room {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// impl ic_stable_structures::Storable for CustomVec<User> {
//     fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
//         borrow::Cow::Owned(Encode!(self).unwrap())
//     }
//
//     fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
//         Decode!(bytes.as_ref(), Self).unwrap()
//     }
// }
//
// impl ic_stable_structures::BoundedStorable for CustomVec<User> {
//     const MAX_SIZE: u32 = 1024;
//     const IS_FIXED_SIZE: bool = false;
// }

#[derive(candid::CandidType, candid::Deserialize)]
pub struct GetRoomByNumberPayload {
    pub number: u64,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct CreateRoomPayload {
    pub number: u64,
    pub capacity: u64,
    pub price_per_occupant: u64,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct BookRoomPayload {
    pub number: u64,
    pub price: u64,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct UnbookRoomPayload {
    pub number: u64,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct DeleteRoomPayload {
    pub number: u64,
}
