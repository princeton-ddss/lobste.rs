use std::io::{Read, Result, Seek, SeekFrom};

use super::{read_kind, read_nanoseconds, read_price, read_refno, read_shares};
use super::{AddOrder, Context, DeleteOrder, Version};

pub(crate) fn read_replace_order<T>(
    buffer: &mut T,
    version: &Version,
    context: &mut Context,
) -> Result<(DeleteOrder, AddOrder)>
where
    T: Read + Seek,
{
    // Read data from buffer
    let _kind = read_kind(buffer)?;
    if version == &Version::V50 {
        buffer.seek(SeekFrom::Current(4))?; // Discard stock locate and tracking number
    }
    let nanoseconds = read_nanoseconds(buffer, version, context.clock)?;
    let old_refno = read_refno(buffer)?;
    let new_refno = read_refno(buffer)?;
    let new_shares = read_shares(buffer)?;
    let new_price = read_price(buffer)?;

    // Update context
    let mut order = context
        .active_orders
        .remove(&old_refno)
        .expect("Order not found");
    let ticker = order.ticker.clone();
    let side = order.side;
    let old_price = order.price;
    let old_shares = order.shares;
    order.price = new_price;
    order.shares = new_shares;
    context.active_orders.insert(new_refno, order);

    // Split the replacement order into delete and add parts
    let delete_order = DeleteOrder::new(
        nanoseconds,
        'D', // `kind`
        ticker.clone(),
        side,
        old_price,
        old_shares,
        old_refno,
        Some(true), // `from_replace`
    );
    let add_order = AddOrder::new(
        nanoseconds,
        'A', // `kind`
        ticker.clone(),
        side,
        new_price,
        new_shares,
        new_refno,
        Some(true), // `from_replace`
        None,       // `mpid`
    );

    // Return messages
    Ok((delete_order, add_order))
}
