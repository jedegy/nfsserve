//! RPC message framing and transmission as specified in RFC 1057 section 10.
//!
//! This module implements the Record Marking Standard for sending RPC messages
//! over TCP connections. It provides:
//!
//! - Message fragmentation for large RPC messages
//! - Proper message delimitation in stream-oriented transports
//! - Asynchronous message processing
//! - RPC call dispatching to appropriate protocol handlers
//!
//! The wire protocol implementation handles all the low-level details of:
//! - Reading fragmentary messages and reassembling them
//! - Writing record-marked fragments with appropriate headers
//! - Managing socket communication channels
//! - Processing incoming RPC calls
//!
//! This module is essential for maintaining proper message boundaries in TCP
//! while providing efficient transmission of RPC messages of any size.

use std::io::Cursor;
use std::io::{Read, Write};

use anyhow::anyhow;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::DuplexStream;
use tokio::sync::mpsc;
use tracing::{debug, error, trace, warn};

use crate::protocol::xdr::{self, mount, nfs3, portmap, XDR};
use crate::protocol::{nfs, rpc};

// Information from RFC 5531 (ONC RPC v2)
// https://datatracker.ietf.org/doc/html/rfc5531
// And RFC 1057 (Original RPC)
// https://datatracker.ietf.org/doc/html/rfc1057

/// RPC program number for NFS Access Control Lists
const NFS_ACL_PROGRAM: u32 = 100227;
/// RPC program number for NFS ID Mapping
const NFS_ID_MAP_PROGRAM: u32 = 100270;
/// RPC program number for NFS Metadata
const NFS_METADATA_PROGRAM: u32 = 200024;

/// Processes a single RPC message
///
/// This function forms the core of the RPC message dispatcher. It:
/// 1. Deserializes the incoming RPC message using XDR format
/// 2. Validates the RPC version number (must be version 2)
/// 3. Extracts authentication information if provided
/// 4. Checks for retransmissions to ensure idempotent operation
/// 5. Routes the call to the appropriate protocol handler (NFS, MOUNT, PORTMAP)
/// 6. Tracks transaction completion state
///
/// This implementation follows RFC 1057 section 8 (Authentication) and
/// section 11 (Record Marking Standard) for proper RPC message handling.
///
/// Returns true if a response was sent, false otherwise (for retransmissions).
async fn handle_rpc(
    input: &mut impl Read,
    output: &mut impl Write,
    mut context: rpc::Context,
) -> Result<bool, anyhow::Error> {
    let mut recv = xdr::rpc::rpc_msg::default();
    recv.deserialize(input)?;
    let xid = recv.xid;
    if let xdr::rpc::rpc_body::CALL(call) = recv.body {
        if let xdr::rpc::auth_flavor::AUTH_UNIX = call.cred.flavor {
            let mut auth = xdr::rpc::auth_unix::default();
            auth.deserialize(&mut Cursor::new(&call.cred.body))?;
            context.auth = auth;
        }
        if call.rpcvers != 2 {
            warn!("Invalid RPC version {} != 2", call.rpcvers);
            xdr::rpc::rpc_vers_mismatch(xid).serialize(output)?;
            return Ok(true);
        }

        if context
            .transaction_tracker
            .is_retransmission(xid, &context.client_addr)
        {
            // This is a retransmission
            // Drop the message and return
            debug!(
                "Retransmission detected, xid: {}, client_addr: {}, call: {:?}",
                xid, context.client_addr, call
            );
            return Ok(false);
        }

        let res = {
            if call.prog == nfs3::PROGRAM {
                nfs::v3::handle_nfs(xid, call, input, output, &context).await
            } else if call.prog == portmap::PROGRAM {
                nfs::portmap::handle_portmap(xid, call, input, output, &context)
            } else if call.prog == mount::PROGRAM {
                nfs::mount::handle_mount(xid, call, input, output, &context).await
            } else if call.prog == NFS_ACL_PROGRAM
                || call.prog == NFS_ID_MAP_PROGRAM
                || call.prog == NFS_METADATA_PROGRAM
            {
                trace!("ignoring NFS_ACL packet");
                xdr::rpc::prog_unavail_reply_message(xid).serialize(output)?;
                Ok(())
            } else {
                warn!(
                    "Unknown RPC Program number {} != {}",
                    call.prog,
                    nfs3::PROGRAM
                );
                xdr::rpc::prog_unavail_reply_message(xid).serialize(output)?;
                Ok(())
            }
        }
        .map(|_| true);
        context
            .transaction_tracker
            .mark_processed(xid, &context.client_addr);
        res
    } else {
        error!("Unexpectedly received a Reply instead of a Call");
        Err(anyhow!("Bad RPC Call format"))
    }
}

/// Reads a single record-marked fragment from a stream
///
/// Implements the RFC 1057 section 10 (Record Marking Standard) for TCP transport.
/// The record marking standard addresses the problem of delimiting records in a
/// stream protocol like TCP by prefixing each record with a 4-byte header.
///
/// This function:
/// 1. Reads the 4-byte header from the socket
/// 2. Extracts the fragment length (lower 31 bits) and last-fragment flag (highest bit)
/// 3. Reads exactly that many bytes from the socket
/// 4. Appends the read data to the provided buffer
///
/// Returns true if this was the last fragment in the RPC record, false otherwise.
/// This allows for reassembly of multi-fragment RPC messages.
async fn read_fragment(
    socket: &mut DuplexStream,
    append_to: &mut Vec<u8>,
) -> Result<bool, anyhow::Error> {
    let mut header_buf = [0_u8; 4];
    socket.read_exact(&mut header_buf).await?;
    let fragment_header = u32::from_be_bytes(header_buf);
    let is_last = (fragment_header & (1 << 31)) > 0;
    let length = (fragment_header & ((1 << 31) - 1)) as usize;
    trace!("Reading fragment length:{}, last:{}", length, is_last);
    let start_offset = append_to.len();
    append_to.resize(append_to.len() + length, 0);
    socket.read_exact(&mut append_to[start_offset..]).await?;
    trace!(
        "Finishing Reading fragment length:{}, last:{}",
        length,
        is_last
    );
    Ok(is_last)
}

/// Writes data as record-marked fragments to a TCP stream
///
/// Implements the RFC 1057 section 10 (Record Marking Standard) for TCP transport.
/// This standard enables RPC to utilize TCP as a transport while maintaining proper
/// message boundaries essential for RPC semantics.
///
/// The function:
/// 1. Divides large buffers into manageable fragments (maximum 2GB each)
/// 2. Prefixes each fragment with a 4-byte header
///    - The lower 31 bits contain the fragment length
///    - The highest bit indicates if this is the last fragment (1=last, 0=more)
/// 3. Writes both header and data to the socket
///
/// This ensures reliable transmission of RPC messages over TCP with proper
/// message framing and enables receivers to allocate appropriate buffer space.
pub async fn write_fragment(
    socket: &mut tokio::net::TcpStream,
    buf: &[u8],
) -> Result<(), anyhow::Error> {
    // Maximum fragment size is 2^31 - 1 bytes
    const MAX_FRAGMENT_SIZE: usize = (1 << 31) - 1;

    let mut offset = 0;
    while offset < buf.len() {
        // Calculate the size of this fragment
        let remaining = buf.len() - offset;
        let fragment_size = std::cmp::min(remaining, MAX_FRAGMENT_SIZE);

        // Determine if this is the last fragment
        let is_last = offset + fragment_size >= buf.len();

        // Create the fragment header
        // The highest bit indicates if this is the last fragment
        let fragment_header = if is_last {
            fragment_size as u32 + (1 << 31)
        } else {
            fragment_size as u32
        };

        let header_buf = u32::to_be_bytes(fragment_header);
        socket.write_all(&header_buf).await?;

        trace!(
            "Writing fragment length:{}, last:{}",
            fragment_size,
            is_last
        );
        socket
            .write_all(&buf[offset..offset + fragment_size])
            .await?;

        offset += fragment_size;
    }

    Ok(())
}

pub type SocketMessageType = Result<Vec<u8>, anyhow::Error>;

/// Handles RPC message processing over a TCP connection
///
/// Receives record-marked RPC messages from a TCP stream, processes
/// them asynchronously by dispatching to the appropriate protocol handlers,
/// and manages the response flow. Implements the record marking protocol
/// for reliable message delimitation over TCP.
#[derive(Debug)]
pub struct SocketMessageHandler {
    cur_fragment: Vec<u8>,
    socket_receive_channel: DuplexStream,
    reply_send_channel: mpsc::UnboundedSender<SocketMessageType>,
    context: rpc::Context,
}

impl SocketMessageHandler {
    /// Creates a new SocketMessageHandler instance
    ///
    /// Initializes the handler with the provided RPC context and creates the
    /// necessary communication channels. Returns the handler itself, a duplex
    /// stream for writing to the socket, and a receiver for processed messages.
    ///
    /// This setup enables asynchronous processing of RPC messages.
    pub fn new(
        context: &rpc::Context,
    ) -> (
        Self,
        DuplexStream,
        mpsc::UnboundedReceiver<SocketMessageType>,
    ) {
        let (socksend, sockrecv) = tokio::io::duplex(256000);
        let (msgsend, msgrecv) = mpsc::unbounded_channel();
        (
            Self {
                cur_fragment: Vec::new(),
                socket_receive_channel: sockrecv,
                reply_send_channel: msgsend,
                context: context.clone(),
            },
            socksend,
            msgrecv,
        )
    }

    /// Reads and processes a fragment from the socket
    ///
    /// Reads a single record-marked fragment from the socket and appends it to
    /// the current message buffer. If the fragment is the last one in the record,
    /// spawns a task to process the complete RPC message and prepare a response.
    /// Should be called in a loop to continuously process incoming messages.
    pub async fn read(&mut self) -> Result<(), anyhow::Error> {
        let is_last =
            read_fragment(&mut self.socket_receive_channel, &mut self.cur_fragment).await?;
        if is_last {
            let fragment = std::mem::take(&mut self.cur_fragment);
            let context = self.context.clone();
            let send = self.reply_send_channel.clone();
            tokio::spawn(async move {
                let mut write_buf: Vec<u8> = Vec::new();
                let mut write_cursor = Cursor::new(&mut write_buf);
                let maybe_reply =
                    handle_rpc(&mut Cursor::new(fragment), &mut write_cursor, context).await;
                match maybe_reply {
                    Err(e) => {
                        error!("RPC Error: {:?}", e);
                        let _ = send.send(Err(e));
                    }
                    Ok(true) => {
                        let _ = std::io::Write::flush(&mut write_cursor);
                        let _ = send.send(Ok(write_buf));
                    }
                    Ok(false) => {
                        // do not reply
                    }
                }
            });
        }
        Ok(())
    }
}
