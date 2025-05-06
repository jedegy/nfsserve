use std::io::{Read, Write};

use num_traits::cast::FromPrimitive;
use tracing::error;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, portmap, XDR};

mod get_port;
mod null;

use get_port::pmapproc_getport;
use null::pmapproc_null;

pub fn handle_portmap(
    xid: u32,
    call: xdr::rpc::call_body,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    if call.vers != portmap::VERSION {
        error!(
            "Invalid Portmap Version number {} != {}",
            call.vers,
            portmap::VERSION
        );
        xdr::rpc::prog_mismatch_reply_message(xid, portmap::VERSION).serialize(output)?;
        return Ok(());
    }
    let prog =
        portmap::PortmapProgram::from_u32(call.proc).unwrap_or(portmap::PortmapProgram::INVALID);

    match prog {
        portmap::PortmapProgram::PMAPPROC_NULL => pmapproc_null(xid, input, output)?,
        portmap::PortmapProgram::PMAPPROC_GETPORT => pmapproc_getport(xid, input, output, context)?,
        _ => {
            xdr::rpc::proc_unavail_reply_message(xid).serialize(output)?;
        }
    }
    Ok(())
}
