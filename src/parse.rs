use std::iter::once;
use std::num::NonZeroU64;
use std::ops::RangeFrom;
use nom::{
    Err::Error,
    Err, IResult, InputIter, InputLength, Slice,
};


const RESERVED_ELEMENT_ID: u32 = 0x1F_FF_FF_FF_u32;

pub fn element_id<I>(input: I) -> IResult<I, u32, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let mut iter = input.iter_elements();
    let first_byte = iter.next().ok_or(nom::Err::Failure(()))?;
    
    // Parse length from stream    
    let len = first_byte.leading_zeros();
    if len >= 4 {
        return Err(nom::Err::Failure(()));
    }

    // Parse value from stream
    let mut result = u32::from(first_byte ^ (1 << (7 - len)));
    for _i in 0..len {
        result = (result << 8) | u32::from(iter.next().ok_or(nom::Err::Failure(()))?);
    }
    // corner-case: reserved ID's
    if (result & !(result+1)) == 0 {  // if all non-length bits are 1's
        result = RESERVED_ELEMENT_ID;
    } 

    Ok((input, result))
}


const UNKNOWN_ELEMENT_LEN: NonZeroU64 = unsafe {NonZeroU64::new_unchecked(u64::MAX)};

pub fn element_len<I>(input: I) -> IResult<I, NonZeroU64, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let mut iter = input.iter_elements();
    let first_byte = iter.next().ok_or(nom::Err::Failure(()))?;
    
    // Parse length from stream    
    let len = first_byte.leading_zeros();
    if len == 8 {
        return Err(nom::Err::Failure(()));
    }

    // Parse value from stream
    let mut value = u64::from(first_byte ^ (1 << (7 - len)));
    for _i in 0..len {
        let item = iter.next().ok_or(nom::Err::Failure(()))?;
        value = (value << 8) | u64::from(item);
    }
    // corner case: erroneous zero-length
    let mut result = NonZeroU64::new(value).ok_or(nom::Err::Failure(()))?;
    // corner-case: unknown data sizes
    if (value & !(value+1)) == 0 {  // if all non-length bits are 1's
        result = UNKNOWN_ELEMENT_LEN;
    } 

    Ok((input, result))
}
