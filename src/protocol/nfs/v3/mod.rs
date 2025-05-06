use std::io::{Read, Write};

use num_traits::cast::FromPrimitive;
use tracing::warn;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};

mod access;
mod commit;
mod create;
mod fsinfo;
mod fsstat;
mod getattr;
mod link;
mod lookup;
mod mkdir;
mod mknod;
mod null;
mod pathconf;
mod read;
mod readdir;
mod readdirplus;
mod readlink;
mod remove;
mod rename;
mod setattr;
mod symlink;
mod write;

use access::nfsproc3_access;
use commit::nfsproc3_commit;
use create::nfsproc3_create;
use fsinfo::nfsproc3_fsinfo;
use fsstat::nfsproc3_fsstat;
use getattr::nfsproc3_getattr;
use link::nfsproc3_link;
use lookup::nfsproc3_lookup;
use mkdir::nfsproc3_mkdir;
use mknod::nfsproc3_mknod;
use null::nfsproc3_null;
use pathconf::nfsproc3_pathconf;
use read::nfsproc3_read;
use readdir::nfsproc3_readdir;
use readdirplus::nfsproc3_readdirplus;
use readlink::nfsproc3_readlink;
use remove::nfsproc3_remove;
use rename::nfsproc3_rename;
use setattr::nfsproc3_setattr;
use symlink::nfsproc3_symlink;
use write::nfsproc3_write;

pub async fn handle_nfs(
    xid: u32,
    call: xdr::rpc::call_body,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    if call.vers != nfs3::VERSION {
        warn!(
            "Invalid NFS Version number {} != {}",
            call.vers,
            nfs3::VERSION
        );
        xdr::rpc::prog_mismatch_reply_message(xid, nfs3::VERSION).serialize(output)?;
        return Ok(());
    }
    let prog = nfs3::NFSProgram::from_u32(call.proc).unwrap_or(nfs3::NFSProgram::INVALID);

    match prog {
        nfs3::NFSProgram::NFSPROC3_NULL => nfsproc3_null(xid, input, output)?,
        nfs3::NFSProgram::NFSPROC3_GETATTR => nfsproc3_getattr(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_LOOKUP => nfsproc3_lookup(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_READ => nfsproc3_read(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_FSINFO => nfsproc3_fsinfo(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_ACCESS => nfsproc3_access(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_PATHCONF => {
            nfsproc3_pathconf(xid, input, output, context).await?
        }
        nfs3::NFSProgram::NFSPROC3_FSSTAT => nfsproc3_fsstat(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_READDIR => nfsproc3_readdir(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_READDIRPLUS => {
            nfsproc3_readdirplus(xid, input, output, context).await?
        }
        nfs3::NFSProgram::NFSPROC3_WRITE => nfsproc3_write(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_CREATE => nfsproc3_create(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_SETATTR => nfsproc3_setattr(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_REMOVE => nfsproc3_remove(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_RMDIR => nfsproc3_remove(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_RENAME => nfsproc3_rename(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_MKDIR => nfsproc3_mkdir(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_SYMLINK => nfsproc3_symlink(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_READLINK => {
            nfsproc3_readlink(xid, input, output, context).await?
        }
        nfs3::NFSProgram::NFSPROC3_MKNOD => nfsproc3_mknod(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_LINK => nfsproc3_link(xid, input, output, context).await?,
        nfs3::NFSProgram::NFSPROC3_COMMIT => nfsproc3_commit(xid, input, output, context).await?,
        _ => {
            warn!("Unimplemented message {:?}", prog);
            xdr::rpc::proc_unavail_reply_message(xid).serialize(output)?;
        } /*
          INVALID*/
    }
    Ok(())
}
