use std::io::{Read, Write};

use tracing::{debug, error, warn};

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};
use crate::vfs;

pub async fn nfsproc3_setattr(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    if !matches!(context.vfs.capabilities(), vfs::Capabilities::ReadWrite) {
        warn!("No write capabilities.");
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        nfs3::nfsstat3::NFS3ERR_ROFS.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        return Ok(());
    }
    let mut args = nfs3::SETATTR3args::default();
    args.deserialize(input)?;
    debug!("nfsproc3_setattr({:?},{:?}) ", xid, args);

    let id = context.vfs.fh_to_id(&args.object);
    // fail if unable to convert file handle
    if let Err(stat) = id {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        return Ok(());
    }
    let id = id.unwrap();

    let ctime;

    let pre_op_attr = match context.vfs.getattr(id).await {
        Ok(v) => {
            let wccattr = nfs3::wcc_attr {
                size: v.size,
                mtime: v.mtime,
                ctime: v.ctime,
            };
            ctime = v.ctime;
            nfs3::pre_op_attr::attributes(wccattr)
        }
        Err(stat) => {
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            nfs3::wcc_data::default().serialize(output)?;
            return Ok(());
        }
    };
    // handle the guard
    match args.guard {
        nfs3::sattrguard3::Void => {}
        nfs3::sattrguard3::obj_ctime(c) => {
            if c.seconds != ctime.seconds || c.nseconds != ctime.nseconds {
                xdr::rpc::make_success_reply(xid).serialize(output)?;
                nfs3::nfsstat3::NFS3ERR_NOT_SYNC.serialize(output)?;
                nfs3::wcc_data::default().serialize(output)?;
            }
        }
    }

    match context.vfs.setattr(id, args.new_attribute).await {
        Ok(post_op_attr) => {
            debug!(" setattr success {:?} --> {:?}", xid, post_op_attr);
            let wcc_res = nfs3::wcc_data {
                before: pre_op_attr,
                after: nfs3::post_op_attr::attributes(post_op_attr),
            };
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            wcc_res.serialize(output)?;
        }
        Err(stat) => {
            error!("setattr error {:?} --> {:?}", xid, stat);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            nfs3::wcc_data::default().serialize(output)?;
        }
    }
    Ok(())
}
