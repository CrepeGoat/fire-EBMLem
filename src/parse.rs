use std::mem::size_of;
use std::ops::RangeFrom;
use std::vec::Vec;

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


const UNKNOWN_ELEMENT_LEN: u64 = u64::MAX;

pub fn element_len<I>(input: I) -> IResult<I, u64, ()>
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
    let mut result = u64::from(first_byte ^ (1 << (7 - len)));
    for _i in 0..len {
        let item = iter.next().ok_or(nom::Err::Failure(()))?;
        result = (result << 8) | u64::from(item);
    }
    // corner-case: unknown data sizes
    if (result & !(result+1)) == 0 {  // if all non-length bits are 1's
        result = UNKNOWN_ELEMENT_LEN;
    } 

    Ok((input, result))
}


fn parse_length<I>(input: I, buffer: &mut [u8]) -> IResult<I, (), ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let mut item_iter = input.iter_elements();
    for buffer_item in buffer.iter_mut() {
        *buffer_item = item_iter.next().ok_or(nom::Err::Failure(()))?;
    }

    Ok((input, ()))
}


pub fn uint<I>(input: I, length: usize) -> IResult<I, u64, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    assert!(1 <= length);
    assert!(length <= size_of::<u64>(), format!(
        "invalid length for UInt (expected n<{:?}, found {:?})",
        size_of::<u64>(), length,
    ));

    let mut buffer = [0u8; size_of::<u64>()];
    let (input, _) = parse_length(input, &mut buffer[(size_of::<u64>()-length)..])?;

    Ok((input, u64::from_be_bytes(buffer)))
}


pub fn int<I>(input: I, length: usize) -> IResult<I, i64, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    assert!(1 <= length);
    assert!(length <= size_of::<i64>(), format!(
        "invalid length for Int (expected n<{:?}, found {:?})",
        size_of::<i64>(), length,
    ));

    let mut buffer = [0u8; size_of::<i64>()];
    let i0 = buffer.len() - length;
    let (input, _) = parse_length(input, &mut buffer[i0..])?;
    // Move the negative bit to the right spot
    if i0 > 0 {
        buffer[0] |= buffer[i0] & 0x80;
        buffer[i0] &= 0x7F;
    }

    Ok((input, i64::from_be_bytes(buffer)))
}

pub fn float32<I>(input: I, length: usize) -> IResult<I, f32, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    assert!(length <= size_of::<f32>(), format!(
        "invalid length for f32 (expected {:?}, found {:?})",
        size_of::<f32>(), length,
    ));

    let mut buffer = [0u8; size_of::<f32>()];
    let (input, _) = parse_length(input, &mut buffer)?;

    Ok((input, f32::from_be_bytes(buffer)))
}


pub fn float64<I>(input: I, length: usize) -> IResult<I, f64, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    assert!(length <= size_of::<f64>(), format!(
        "invalid length for f64 (expected {:?}, found {:?})",
        size_of::<f64>(), length,
    ));

    let mut buffer = [0u8; size_of::<f64>()];
    let (input, _) = parse_length(input, &mut buffer)?;

    Ok((input, f64::from_be_bytes(buffer)))
}


pub fn ascii_str<I>(input: I, length: usize) -> IResult<I, String, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (input, result) = unicode_str(input, length)?;
    if !result[..].is_ascii() {
        return Err(nom::Err::Failure(()));
    }

    Ok((input, result))
}


pub fn unicode_str<I>(input: I, length: usize) -> IResult<I, String, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let mut buffer = vec![0u8; length];
    let (input, _) = parse_length(input, &mut buffer[..])?;

    let result = String::from_utf8(buffer).or(Err(nom::Err::Failure(())))?;

    Ok((input, result))
}


pub fn date<I>(input: I, length: usize) -> IResult<I, i64, ()>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    assert!(length <= size_of::<u64>(), format!(
        "invalid length for timestamp (expected {:?}, found {:?})",
        size_of::<u64>(), length,
    ));

    let mut buffer = [0u8; size_of::<i64>()];
    let (input, _) = parse_length(input, &mut buffer)?;

    Ok((input, i64::from_be_bytes(buffer)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_id() {
        
    }
}
