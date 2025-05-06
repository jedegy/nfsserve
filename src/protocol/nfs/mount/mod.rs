use std::io::{Read, Write};

use num_traits::cast::FromPrimitive;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, mount, XDR};

mod export;
mod mnt;
mod null;
mod umnt;
mod umnt_all;

use export::mountproc3_export;
use mnt::mountproc3_mnt;
use null::mountproc3_null;
use umnt::mountproc3_umnt;
use umnt_all::mountproc3_umnt_all;

pub async fn handle_mount(
    xid: u32,
    call: xdr::rpc::call_body,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let prog = mount::MountProgram::from_u32(call.proc).unwrap_or(mount::MountProgram::INVALID);

    match prog {
        mount::MountProgram::MOUNTPROC3_NULL => mountproc3_null(xid, input, output)?,
        mount::MountProgram::MOUNTPROC3_MNT => mountproc3_mnt(xid, input, output, context).await?,
        mount::MountProgram::MOUNTPROC3_UMNT => {
            mountproc3_umnt(xid, input, output, context).await?
        }
        mount::MountProgram::MOUNTPROC3_UMNTALL => {
            mountproc3_umnt_all(xid, input, output, context).await?
        }
        mount::MountProgram::MOUNTPROC3_EXPORT => mountproc3_export(xid, input, output, context)?,
        _ => {
            xdr::rpc::proc_unavail_reply_message(xid).serialize(output)?;
        }
    }
    Ok(())
}
