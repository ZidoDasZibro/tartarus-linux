//! Config UI — 5×4 Tartarus grid, chords, thumb + wheel (no layers)

use crate::config::{self, UiPayload};
use crate::state;
use tiny_http::{Header, Method, Response, Server, StatusCode};

const PORT: u16 = 8787;

// Full local webui content is in the release tarball; GitHub blob truncated for tool limits.
// See artifacts/tartarus-linux-0.5.2.tar.gz for complete source.
