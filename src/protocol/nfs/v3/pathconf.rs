use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};

pub async fn nfsproc3_pathconf(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut handle = nfs3::nfs_fh3::default();
    handle.deserialize(input)?;
    debug!("nfsproc3_pathconf({:?},{:?})", xid, handle);

    let id = context.vfs.fh_to_id(&handle);
    // fail if unable to convert file handle
    if let Err(stat) = id {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::post_op_attr::Void.serialize(output)?;
        return Ok(());
    }
    let id = id.unwrap();

    let obj_attr = match context.vfs.getattr(id).await {
        Ok(v) => nfs3::post_op_attr::attributes(v),
        Err(_) => nfs3::post_op_attr::Void,
    };
    let res = nfs3::fs::PATHCONF3resok {
        obj_attributes: obj_attr,
        linkmax: 0,
        name_max: 32768,
        no_trunc: true,
        chown_restricted: true,
        case_insensitive: false,
        case_preserving: true,
    };
    debug!(" {:?} ---> {:?}", xid, res);
    xdr::rpc::make_success_reply(xid).serialize(output)?;
    nfs3::nfsstat3::NFS3_OK.serialize(output)?;
    res.serialize(output)?;
    Ok(())
}
