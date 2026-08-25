//! Config UI — 5×4 Tartarus grid, chords, thumb + wheel (no layers)

use crate::config::{self, UiPayload};
use crate::state;
use tiny_http::{Header, Method, Response, Server, StatusCode};

const PORT: u16 = 8787;

// Content restored from local 0.5.1 release; full source in tartarus-linux-0.5.1.tar.gz
