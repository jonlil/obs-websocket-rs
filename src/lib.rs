#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod events;
pub mod requests;
pub mod responses;
mod session;
mod web_socket;

pub use events::ObsEvent;
pub use requests::RequestType;
pub use session::ObsSession;
pub use web_socket::{ObsEventEmitter, ObsResponse, ObsWebSocket};

pub fn connect(address: &str, password: String) -> ObsSession {
    let obs = ObsWebSocket::connect(address, password);
    ObsSession::new(obs)
}
