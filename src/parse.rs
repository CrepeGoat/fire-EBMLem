use std::mem::size_of;
use std::ops::RangeFrom;
use std::vec::Vec;

use nom::{
    Err::Error,
    Err, IResult, InputIter, InputLength, Slice,
};


const RESERVED_ELEMENT_ID: u32 = 0x1F_FF_FF_FF_u32;

pub fn element_id(input: &[u8]) -> IResult<&[u8], u32, ()>
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

    Ok((&input[((len+1) as usize)..], result))
}


const UNKNOWN_ELEMENT_LEN: u64 = u64::MAX;

pub fn element_len(input: &[u8]) -> IResult<&[u8], u64, ()>
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

    Ok((&input[((len+1) as usize)..], result))
}


fn parse_length<'a>(input: &'a[u8], buffer: &mut [u8]) -> IResult<&'a[u8], (), ()>
{
    let mut item_iter = input.iter_elements();
    for buffer_item in buffer.iter_mut() {
        *buffer_item = item_iter.next().ok_or(nom::Err::Failure(()))?;
    }

    Ok((&input[buffer.len()..], ()))
}


pub fn uint(input: &[u8], length: usize) -> IResult<&[u8], u64, ()>
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


pub fn int(input: &[u8], length: usize) -> IResult<&[u8], i64, ()>
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

pub fn float32(input: &[u8], length: usize) -> IResult<&[u8], f32, ()>
{
    assert!(length <= size_of::<f32>(), format!(
        "invalid length for f32 (expected {:?}, found {:?})",
        size_of::<f32>(), length,
    ));

    let mut buffer = [0u8; size_of::<f32>()];
    let (input, _) = parse_length(input, &mut buffer)?;

    Ok((input, f32::from_be_bytes(buffer)))
}


pub fn float64(input: &[u8], length: usize) -> IResult<&[u8], f64, ()>
{
    assert!(length <= size_of::<f64>(), format!(
        "invalid length for f64 (expected {:?}, found {:?})",
        size_of::<f64>(), length,
    ));

    let mut buffer = [0u8; size_of::<f64>()];
    let (input, _) = parse_length(input, &mut buffer)?;

    Ok((input, f64::from_be_bytes(buffer)))
}


pub fn ascii_str(input: &[u8], length: usize) -> IResult<&[u8], &str, ()>
{
    let (input, result) = unicode_str(input, length)?;
    if !result[..].is_ascii() {
        return Err(nom::Err::Failure(()));
    }

    Ok((input, result))
}


pub fn unicode_str(input: &[u8], length: usize) -> IResult<&[u8], &str, ()>
{
    if input.len() < length {
        return Err(nom::Err::Failure(()));
    }
    let result = std::str::from_utf8(&input[..length]).or(Err(nom::Err::Failure(())))?;

    Ok((&input[length..], result))
}


pub fn date(input: &[u8], length: usize) -> IResult<&[u8], i64, ()>
{
    assert!(length <= size_of::<i64>(), format!(
        "invalid length for timestamp (expected {:?}, found {:?})",
        size_of::<i64>(), length,
    ));

    let mut buffer = [0u8; size_of::<i64>()];
    let (input, _) = parse_length(input, &mut buffer)?;

    Ok((input, i64::from_be_bytes(buffer)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_id() {
        let source = [0x40, 0x01, 0xFF];
        assert_eq!(element_id(&source[..]), Ok((&source[2..], 1)));
    }

    #[test]
    fn test_element_len() {
        let source = [0x40, 0x01, 0xFF];
        assert_eq!(element_len(&source[..]), Ok((&source[2..], 1)));
    }

    #[test]
    fn test_uint() {
        let source = [0x40, 0x01, 0xFF];
        assert_eq!(uint(&source[..], 1), Ok((&source[1..], source[0] as u64)));
    }

    #[test]
    fn test_int() {
        let source = [0x40, 0x01, 0xFF];
        assert_eq!(int(&source[..], 1), Ok((&source[1..], i8::from_be_bytes([source[0]]) as i64)));
    }

    #[test]
    fn test_float32() {
        let num = 3.0f32;
        let source = num.to_be_bytes();
        assert_eq!(float32(&source[..], 4), Ok((&source[4..], num)));
    }

    #[test]
    fn test_float64() {
        let num = 5.0f64;
        let source = num.to_be_bytes();
        assert_eq!(float64(&source[..], 8), Ok((&source[8..], num)));
    }

    #[test]
    fn test_ascii_str() {
        let source = b"I am a string, I am only a string.";
        assert_eq!(ascii_str(&source[..], 8), Ok((&source[8..], "I am a s")));
    }

    #[test]
    fn test_unicode_str() {
        let s = "You do say the strangest of things, mein Fräulein.";
        let source = s.as_bytes();
        assert_eq!(unicode_str(&source[36..], 11), Ok((&source[47..], "mein Fräul")));
    }

    #[test]
    fn test_date() {
        let source = [0x40, 0x01, 0xFF, 0x00, 0x40, 0x01, 0xFF, 0x00, 0xFF, 0xFF];
        assert_eq!(
            date(&source[..], 1),
            Ok((&source[8..], i64::from_be_bytes(
                [0x40, 0x01, 0xFF, 0x00, 0x40, 0x01, 0xFF, 0x00],
            ))),
        );
    }
}
