use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_kind, read_nanoseconds, read_refno, read_shares};
use super::{Context, ReadMessage, Side, Version};
use super::{IntoOrderMessage, OrderMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct CancelOrder {
    nanoseconds: u64,
    kind: char,
    ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    refno: u64,
}

impl ReadMessage for CancelOrder {
    fn read<T>(buffer: &mut T, version: &Version, context: &mut Context) -> Result<Self>
    where
        T: Read + Seek,
    {
        // Read data from buffer
        let kind = read_kind(buffer)?;
        if version == &Version::V50 {
            buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
        }
        let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
        let refno = read_refno(buffer)?;
        let shares = read_shares(buffer)?;

        // Update context
        let order = context
            .active_orders
            .get_mut(&refno)
            .expect("Order not found");
        order.shares -= shares;

        // Return message
        Ok(Self {
            nanoseconds,
            kind,
            ticker: order.ticker.clone(),
            side: order.side,
            price: order.price,
            shares,
            refno,
        })
    }
}

impl IntoOrderMessage for CancelOrder {
    fn into_order_message(self, date: String) -> OrderMessage {
        OrderMessage {
            date,
            nanoseconds: self.nanoseconds,
            kind: self.kind,
            ticker: self.ticker,
            side: self.side,
            price: self.price,
            shares: self.shares,
            refno: self.refno,
            from_replace: None,
            mpid: None,
            printable: None,
            execution_price: None,
        }
    }
}
