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

pub fn connect<T: ObsEventEmitter + Send + 'static>(
    address: &str,
    password: String,
    tx: T,
) -> Result<ObsSession, &'static str> {
    let obs = ObsWebSocket::connect(address, password, Box::new(tx))?;

    Ok(ObsSession::new(obs))
}
