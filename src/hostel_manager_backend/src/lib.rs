use candid::{Decode, Encode};
use ic_stable_structures::memory_manager;
use std::{cell, sync::{Arc, Mutex}};

type Memory =
    ic_stable_structures::memory_manager::VirtualMemory<ic_stable_structures::DefaultMemoryImpl>;
type IdCell = ic_stable_structures::Cell<u64, Memory>;

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize)]
enum Error {
    RoomNotFound,
    RoomNotAvailable,
    RoomFull,
    RoomAlreadyBooked,
    InsufficientPrice,
    NotOwner,
    NotInRoom,
    InvalidCapacity,
}

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
struct User(String);

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
enum RoomState {
    Full,
    PartiallyVacant,
    TotallyVacant,
}

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone)]
struct Room {
    no: u64,
    capacity: usize,
    price_per_occupant: usize,
    state: RoomState,
    occupants: Vec<User>,
    owner: User,
}

impl ic_stable_structures::Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::BoundedStorable for User {
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

lazy_static::lazy_static! {
    static ref MEMORY_MANAGER: Mutex<memory_manager::MemoryManager<ic_stable_structures::DefaultMemoryImpl>> =
        Mutex::new(memory_manager::MemoryManager::init(ic_stable_structures::DefaultMemoryImpl::default()).expect("Failed to initialize Memory Manager"));
    static ref ROOM_COUNTER: IdCell = IdCell::init(
        MEMORY_MANAGER.lock().unwrap().get(memory_manager::MemoryId::new(0)),
        1,
    )
    .expect("Failed to initialize ROOM counter");
    static ref ROOMS: Mutex<Vec<Room>> = Mutex::new(vec![]);
}

#[ic_cdk::query]
fn get_rooms() -> Vec<Room> {
    ROOMS.lock().unwrap().clone()
}

#[ic_cdk::query]
fn get_room_by_number(room_no: u64) -> Result<Room, Error> {
    let rooms = ROOMS.lock().unwrap();
    match rooms.iter().find(|r| r.no == room_no) {
        Some(room) => Ok(room.clone()),
        None => Err(Error::RoomNotFound),
    }
}

#[ic_cdk::update]
fn create_room(capacity: usize, price_per_occupant: usize) -> Result<(), Error> {
    if capacity <= 0 {
        return Err(Error::InvalidCapacity);
    }

    let room_number = ROOM_COUNTER.get();
    let room = Room {
        no: room_number,
        capacity,
        price_per_occupant,
        state: RoomState::TotallyVacant,
        occupants: Vec::new(),
        owner: User(ic_cdk::caller().to_string()),
    };

    ROOMS.lock().unwrap().push(room);
    Ok(())
}

#[ic_cdk::update]
fn book_room(room_no: u64, price: usize) -> Result<(), Error> {
    let mut rooms = ROOMS.lock().unwrap();
    let room = match rooms.iter_mut().find(|r| r.no == room_no) {
        Some(room) => room,
        None => return Err(Error::RoomNotAvailable),
    };

    if room.state == RoomState::Full {
        return Err(Error::RoomFull);
    }

    if price < room.price_per_occupant {
        return Err(Error::InsufficientPrice);
    }

    let occupant = User(ic_cdk::caller().to_string());

    if room.occupants.iter().any(|o| *o == occupant) {
        return Err(Error::RoomAlreadyBooked);
    }

    room.occupants.push(occupant);
    Ok(())
}

#[ic_cdk::update]
fn unbook_room(room_no: u64) -> Result<(), Error> {
    let mut rooms = ROOMS.lock().unwrap();
    let room = match rooms.iter_mut().find(|r| r.no == room_no) {
        Some(room) => room,
        None => return Err(Error::RoomNotAvailable),
    };

    let occupant = User(ic_cdk::caller().to_string());

    if let Some(index) = room.occupants.iter().position(|o| *o == occupant) {
        room.occupants.remove(index);
        Ok(())
    } else {
        Err(Error::NotInRoom)
    }
}

#[ic_cdk::update]
fn delete_room(room_no: u64) -> Result<(), Error> {
    let mut rooms = ROOMS.lock().unwrap();
    if let Some(room_index) = rooms.iter().position(|room| room.no == room_no) {
        let room = &rooms[room_index];

        if room.owner != User(ic_cdk::caller().to_string()) {
            return Err(Error::NotOwner);
        }

        rooms.remove(room_index);
        Ok(())
    } else {
        Err(Error::RoomNotAvailable)
    }
}

ic_cdk::export_candid!();
