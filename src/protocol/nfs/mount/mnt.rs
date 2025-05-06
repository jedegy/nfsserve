use std::io::{Read, Write};

use num_traits::cast::ToPrimitive;
use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, mount, XDR};

pub async fn mountproc3_mnt(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut path = mount::dirpath::new();
    path.deserialize(input)?;
    let utf8path = std::str::from_utf8(&path).unwrap_or_default();
    debug!("mountproc3_mnt({:?},{:?}) ", xid, utf8path);
    let path = if let Some(path) = utf8path.strip_prefix(context.export_name.as_str()) {
        let path = path
            .trim_start_matches('/')
            .trim_end_matches('/')
            .trim()
            .as_bytes();
        let mut new_path = Vec::with_capacity(path.len() + 1);
        new_path.push(b'/');
        new_path.extend_from_slice(path);
        new_path
    } else {
        // invalid export
        debug!("{:?} --> no matching export", xid);
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        mount::mountstat3::MNT3ERR_NOENT.serialize(output)?;
        return Ok(());
    };
    if let Ok(fileid) = context.vfs.path_to_id(&path).await {
        let response = mount::mountres3_ok {
            fhandle: context.vfs.id_to_fh(fileid).data,
            auth_flavors: vec![
                xdr::rpc::auth_flavor::AUTH_NULL.to_u32().unwrap(),
                xdr::rpc::auth_flavor::AUTH_UNIX.to_u32().unwrap(),
            ],
        };
        debug!("{:?} --> {:?}", xid, response);
        if let Some(ref chan) = context.mount_signal {
            let _ = chan.send(true).await;
        }
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        mount::mountstat3::MNT3_OK.serialize(output)?;
        response.serialize(output)?;
    } else {
        debug!("{:?} --> MNT3ERR_NOENT", xid);
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        mount::mountstat3::MNT3ERR_NOENT.serialize(output)?;
    }
    Ok(())
}
