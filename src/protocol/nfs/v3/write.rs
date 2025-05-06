use std::io::{Read, Write};

use tracing::{debug, error, warn};

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};
use crate::vfs;

pub async fn nfsproc3_write(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    // if we do not have write capabilities
    if !matches!(context.vfs.capabilities(), vfs::Capabilities::ReadWrite) {
        warn!("No write capabilities.");
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        nfs3::nfsstat3::NFS3ERR_ROFS.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        return Ok(());
    }

    let mut args = nfs3::file::WRITE3args::default();
    args.deserialize(input)?;
    debug!("nfsproc3_write({:?},...) ", xid);
    // sanity check the length
    if args.data.len() != args.count as usize {
        xdr::rpc::garbage_args_reply_message(xid).serialize(output)?;
        return Ok(());
    }

    let id = context.vfs.fh_to_id(&args.file);
    if let Err(stat) = id {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        return Ok(());
    }
    let id = id.unwrap();

    // get the object attributes before the write
    let pre_obj_attr = match context.vfs.getattr(id).await {
        Ok(v) => {
            let wccattr = nfs3::wcc_attr {
                size: v.size,
                mtime: v.mtime,
                ctime: v.ctime,
            };
            nfs3::pre_op_attr::attributes(wccattr)
        }
        Err(_) => nfs3::pre_op_attr::Void,
    };

    match context.vfs.write(id, args.offset, &args.data).await {
        Ok(fattr) => {
            debug!("write success {:?} --> {:?}", xid, fattr);
            let res = nfs3::file::WRITE3resok {
                file_wcc: nfs3::wcc_data {
                    before: pre_obj_attr,
                    after: nfs3::post_op_attr::attributes(fattr),
                },
                count: args.count,
                committed: nfs3::file::stable_how::FILE_SYNC,
                verf: context.vfs.serverid(),
            };
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            res.serialize(output)?;
        }
        Err(stat) => {
            error!("write error {:?} --> {:?}", xid, stat);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            nfs3::wcc_data::default().serialize(output)?;
        }
    }
    Ok(())
}
