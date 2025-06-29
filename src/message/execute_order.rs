use std::io::{Read, Result, Seek, SeekFrom};

use getset::Getters;

use super::{read_kind, read_nanoseconds, read_price, read_printable, read_refno, read_shares};
use super::{Context, ReadMessage, Side, Version};
use super::{IntoOrderMessage, OrderMessage};

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ExecuteOrder {
    nanoseconds: u64,
    kind: char,
    ticker: String,
    side: Side,
    price: u32,
    shares: u32,
    refno: u64,
    printable: Option<bool>,
    execution_price: Option<u32>,
}

impl ReadMessage for ExecuteOrder {
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
        buffer.seek(SeekFrom::Current(8))?; // Discard match number
        let (printable, execution_price) = if kind == 'C' {
            let printable = Some(read_printable(buffer)?);
            let execution_price = Some(read_price(buffer)?);
            (printable, execution_price)
        } else {
            (None, None)
        };

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
            printable,
            execution_price,
        })
    }
}

impl IntoOrderMessage for ExecuteOrder {
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
            printable: self.printable,
            execution_price: self.execution_price,
        }
    }
}
