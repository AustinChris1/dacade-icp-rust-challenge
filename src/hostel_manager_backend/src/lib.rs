use candid::{Decode, Encode};
use ic_stable_structures::memory_manager;
use std::{borrow, cell};

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
    NotInRoom
}

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
struct User(String);

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone)]
struct MyVec<T>(Vec<T>);

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
    occupants: MyVec<User>,
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

thread_local! {
    static MEMORY_MANAGER: cell::RefCell<memory_manager::MemoryManager<ic_stable_structures::DefaultMemoryImpl>> = cell::RefCell::new(memory_manager::MemoryManager::init(ic_stable_structures::DefaultMemoryImpl::default()));
    static ROOM_COUNTER: cell::RefCell<IdCell> = cell::RefCell::new(IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(memory_manager::MemoryId::new(0))), 0).expect("Failed to initialize ROOM counter"));
    // static ROOMS: cell::RefCell<StableVec<Room>> = cell::RefCell::new(StableVec::init(MEMORY_MANAGER.with(|m| m.borrow().get(memory_manager::MemoryId::new(1)))).expect("Failed to initialize vector!"));
    static ROOMS: cell::RefCell<MyVec<Room>> = cell::RefCell::new(MyVec(vec![]));
}

#[ic_cdk::query]
fn get_rooms() -> Vec<Room> {
    ROOMS.with(|r| r.borrow().0.clone())
}

#[ic_cdk::query]
fn get_room_by_number(room_no: u64) -> Result<Room, Error> {
    ROOMS.with(|r| {
        let rooms = r.borrow();
        match rooms.0.iter().find(|r| r.no == room_no) {
            Some(room) => Ok(room.clone()),
            None => Err(Error::RoomNotFound),
        }
    })
}

#[ic_cdk::update]
fn create_room(capacity: usize, price_per_occupant: usize) -> () {
    ROOM_COUNTER.with(|c| {
        let room_number = *c.borrow().get();
        let room = Room {
            no: room_number,
            capacity,
            price_per_occupant,
            state: RoomState::TotallyVacant,
            occupants: MyVec(vec![]),
            owner: User(ic_cdk::caller().to_string()),
        };
        c.borrow_mut().set(room_number + 1).expect("Failed to increment counter.");
        ROOMS.with(|r| r.borrow_mut().0.push(room));
    })
}

#[ic_cdk::update]
fn book_room(room_no: u64, price: usize) -> Result<(), Error> {
    ROOMS.with(|r| {
        let mut rooms = r.borrow_mut();
        let room = match rooms.0.iter_mut().find(|r| r.no == room_no) {
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

        match room.occupants.0.iter().find(|o| **o == occupant) {
            Some(_) => return Err(Error::RoomAlreadyBooked),
            None => {    
                room.occupants.0.push(occupant);
                if room.occupants.0.len() == room.capacity {
                    room.state = RoomState::Full;
                }
                Ok(())
            },
        }
    })
}

#[ic_cdk::update]
fn unbook_room(room_no: u64) -> Result<(), Error> {
    ROOMS.with(|r| {
        let mut rooms = r.borrow_mut();
        let room = match rooms.0.iter_mut().find(|r| r.no == room_no) {
            Some(room) => room,
            None => return Err(Error::RoomNotAvailable),
        };

        let occupant = User(ic_cdk::caller().to_string());

        match room.occupants.0.iter().position(|o| *o == occupant) {
            Some(index) => {
                room.occupants.0.remove(index);
                if room.state == RoomState::Full {
                    room.state = RoomState::PartiallyVacant;
                }
                Ok(())
            }
            None => Err(Error::NotInRoom),
        }
    })
}

#[ic_cdk::update]
fn delete_room(room_no: u64) -> Result<(), Error> {
    ROOMS.with(|r| {
        let mut rooms = r.borrow_mut();
        let room_index = rooms
            .0
            .iter()
            .position(|room| room.no == room_no)
            .ok_or(Error::RoomNotAvailable)?;

        let room = &rooms.0[room_index];

        if room.owner != User(ic_cdk::caller().to_string()) {
            return Err(Error::NotOwner);
        }

        rooms.0.remove(room_index);

        Ok(())
    })
}

ic_cdk::export_candid!();
