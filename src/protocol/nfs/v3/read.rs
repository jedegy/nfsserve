use std::io::{Read, Write};

use tracing::{debug, error};

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};

pub async fn nfsproc3_read(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut args = nfs3::file::READ3args::default();
    args.deserialize(input)?;
    debug!("nfsproc3_read({:?},{:?}) ", xid, args);

    let id = context.vfs.fh_to_id(&args.file);
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
    match context.vfs.read(id, args.offset, args.count).await {
        Ok((bytes, eof)) => {
            let res = nfs3::file::READ3resok {
                file_attributes: obj_attr,
                count: bytes.len() as u32,
                eof,
                data: bytes,
            };
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            res.serialize(output)?;
        }
        Err(stat) => {
            error!("nfsproc3_read error {:?} --> {:?}", xid, stat);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            obj_attr.serialize(output)?;
        }
    }
    Ok(())
}
