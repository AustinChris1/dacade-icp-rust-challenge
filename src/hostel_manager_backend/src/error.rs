#[derive(candid::CandidType, serde::Serialize, serde::Deserialize)]
pub enum Error {
    RoomNotFound,
    RoomNotAvailable,
    RoomFull,
    RoomAlreadyBooked,
    NotOwner,
    NotInRoom,
    RoomAlreadyExists,
    InvalidPrice,
}

