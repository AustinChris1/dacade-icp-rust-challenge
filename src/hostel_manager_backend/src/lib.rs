use ic_stable_structures::memory_manager;
// use ic_stable_structures::StableBTreeMap;
use std::cell;
use std::collections::HashMap;

mod error;
mod models;

type Memory = memory_manager::VirtualMemory<ic_stable_structures::DefaultMemoryImpl>;
// type IdCell = ic_stable_structures::Cell<u64, Memory>;

thread_local! {
    static MEMORY_MANAGER: cell::RefCell<memory_manager::MemoryManager<ic_stable_structures::DefaultMemoryImpl>> = cell::RefCell::new(memory_manager::MemoryManager::init(ic_stable_structures::DefaultMemoryImpl::default()));
    // static ROOM_COUNTER: cell::RefCell<IdCell> = cell::RefCell::new(IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(memory_manager::MemoryId::new(0))), 1).expect("Failed to initialize ROOM counter"));
    // static ROOMS: cell::RefCell<StableVec<Room>> = cell::RefCell::new(StableVec::init(MEMORY_MANAGER.with(|m| m.borrow().get(memory_manager::MemoryId::new(1)))).expect("Failed to initialize vector!"));
    // static ROOMS: cell::RefCell<StableBTreeMap<u64, models::Room, Memory>> = cell::RefCell::new(
    //     StableBTreeMap::init(
    //         MEMORY_MANAGER.with(|m| m.borrow().get(memory_manager::MemoryId::new(1)))
    //     )
    // );
    static ROOMS: cell::RefCell<HashMap<u64, models::Room>> = cell::RefCell::new(
        HashMap::new()
    );
}

#[ic_cdk::query]
fn get_rooms() -> Vec<models::Room> {
    ROOMS.with(|r| r.borrow().iter().map(|(_, room)| room.clone()).collect())
}

#[ic_cdk::query]
fn get_room_by_number(
    payload: models::GetRoomByNumberPayload,
) -> Result<models::Room, error::Error> {
    ROOMS.with(|r| {
        let rooms = r.borrow();
        let room = rooms
            .get(&payload.number)
            .ok_or(error::Error::RoomNotFound)?;
        Ok(room.clone())
    })
}

#[ic_cdk::update]
fn create_room(payload: models::CreateRoomPayload) -> Result<String, error::Error> {
    let room = models::Room::new(
        payload.number,
        payload.capacity,
        payload.price_per_occupant,
        models::Occupant::new(ic_cdk::caller())
    );

    ROOMS.with(|r| {
        let mut rooms = r.borrow_mut();
        if rooms.contains_key(&room.no) {
            return Err(error::Error::RoomAlreadyExists);
        }
        rooms.insert(room.no, room);
        Ok(String::from("Room created successfully!"))
    })
}

#[ic_cdk::update]
fn book_room(payload: models::BookRoomPayload) -> Result<String, error::Error> {
    ROOMS.with(|r| {
        let mut rooms = r.borrow_mut();
        let room = rooms
            .get_mut(&payload.number)
            .ok_or(error::Error::RoomNotFound)?;

        if room.is_full() {
            return Err(error::Error::RoomFull);
        }

        if !room.check_price(payload.price) {
            return Err(error::Error::InvalidPrice);
        }

        let occupant = models::Occupant::new(ic_cdk::caller());

        match room.add_occupant(occupant) {
            Ok(_) => Ok(String::from("Room successfully booked")),
            Err(err) => Err(err),
        }
    })
}

#[ic_cdk::update]
fn unbook_room(payload: models::UnbookRoomPayload) -> Result<String, error::Error> {
    ROOMS.with(|r| {
        let mut rooms = r.borrow_mut();
        let room = rooms
            .get_mut(&payload.number)
            .ok_or(error::Error::RoomNotFound)?;
        let occupant = models::Occupant::new(ic_cdk::caller());

        match room.has_occupant(occupant.clone()) {
            Some(_) => match room.remove_occupant(occupant) {
                Ok(_) => Ok(String::from("Room unbooked successfully!")),
                Err(_) => Err(error::Error::NotInRoom),
            },
            None => Err(error::Error::NotInRoom),
        }
    })
}

#[ic_cdk::update]
fn delete_room(payload: models::DeleteRoomPayload) -> Result<String, error::Error> {
    ROOMS.with(|r| {
        let mut rooms = r.borrow_mut();
        let room = rooms
            .get(&payload.number)
            .ok_or(error::Error::RoomNotFound)?;

        if !room.is_owner(models::Occupant::new(ic_cdk::caller())) {
            return Err(error::Error::NotOwner);
        }

        rooms.remove(&payload.number);

        Ok(String::from("Room deleted successfully!"))
    })
}

ic_cdk::export_candid!();
