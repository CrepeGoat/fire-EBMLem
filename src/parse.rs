use std::cmp::min;
use std::mem::size_of;
use std::ops::RangeFrom;


use nom::Err;
use nom::Needed;
use nom::error::ParseError;
use nom::ToUsize;
use nom::bits::streaming::{take as take_bits};
use nom::bytes::streaming::{take as take_bytes};


use nom::{
    IResult, InputIter, InputLength, Slice,
};


fn take_rem<I, E: ParseError<(I, usize)>>(
) -> impl Fn((I, usize)) -> IResult<(I, usize), (u8, usize), E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    move |(input, bit_offset): (I, usize)| {
        if bit_offset == 0 {
            Ok(((input, 0), (0u8, 0usize)))
        } else {
            let len = 8 - bit_offset;
            let mut item = match input.iter_elements().next() {
                Some(i) => i,
                None => unreachable!(),  // <- bit-offset != 0 (i.e., they've already pulled 1+ bits from this byte)
            };
            item &= 0xFF >> bit_offset;  // mask out first `bit_offset` bits

            Ok(((input.slice(1..), 0), (item, len as usize)))
        }
    }
}


fn take_zeros<I, C, E: ParseError<(I, usize)>>(
    max_count: C,
) -> impl Fn((I, usize)) -> IResult<(I, usize), usize, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    C: ToUsize,
{
    let max_count = max_count.to_usize();
    move |(mut input, bit_offset): (I, usize)| {
        if max_count == 0 {
            return Ok(((input, bit_offset), 0usize));
        }

        let mut streak_len: usize = 0;
        let mut item = input.iter_elements().next().ok_or_else(|| Err::Incomplete(Needed::new(1)))?;
        item &= 0xFF >> bit_offset;  // mask out first `bit_offset` bits

        streak_len += (item.leading_zeros() as usize) - bit_offset;
        while item.leading_zeros() == 8 && streak_len <= max_count {
            input = input.slice(1..);
            if streak_len == max_count {break};
            item = input.iter_elements().next().ok_or_else(|| Err::Incomplete(Needed::new(1)))?;
            streak_len += item.leading_zeros() as usize;
        }
        streak_len = min(streak_len, max_count);

        Ok(((input, (streak_len + bit_offset) % 8), streak_len))
    }
}


macro_rules! make_vlen_parser {
    ($func_name:ident, $uint:ty) => {
        fn $func_name(input: &[u8]) -> IResult<&[u8], $uint, ()>
        {
            // Parse length from stream
            let ((input, bit_offset), len) = take_zeros(size_of::<$uint>())((input, 0))?;
            if len >= size_of::<$uint>() {
                return Err(nom::Err::Failure(()));
            }
            let ((input, bit_offset), _) = take_bits::<_, usize, _, ()>(1u8)((input, bit_offset))?;
            let ((input, _), (leftover_bits, _)) = take_rem()((input, bit_offset))?;
            let (input, bytes) = take_bytes(len)(input)?;

            let mut buffer = [0u8; size_of::<$uint>()];
            buffer[size_of::<$uint>() - len - 1] = leftover_bits;
            buffer[(size_of::<$uint>() - len)..].copy_from_slice(bytes);

            Ok((input, <$uint>::from_be_bytes(buffer)))
        }
    };
}

make_vlen_parser!(vlen_to_u32, u32);
make_vlen_parser!(vlen_to_u64, u64);


const RESERVED_ELEMENT_ID: u32 = u32::MAX;

pub fn element_id(input: &[u8]) -> IResult<&[u8], u32, ()>
{
    let (new_input, result) = vlen_to_u32(input)?;
    
    let len = unsafe {new_input.as_ptr().offset_from(input.as_ptr())} as u32;
    Ok(
        if result.count_ones() == 7*(len as u32) {  // if all non-length bits are 1's
            // corner-case: reserved ID's
            (new_input, RESERVED_ELEMENT_ID)
        } else {
            (new_input, result)
        }
    )
}


const UNKNOWN_ELEMENT_LEN: u64 = u64::MAX;

pub fn element_len(input: &[u8]) -> IResult<&[u8], u64, ()>
{
    let (new_input, result) = vlen_to_u64(input)?;
    
    let len = unsafe {new_input.as_ptr().offset_from(input.as_ptr())};
    Ok(
        if result == 0xFF && len == 1 {  // if all non-length bits are 1's
            // corner-case: reserved ID's
            (new_input, UNKNOWN_ELEMENT_LEN)
        } else {
            (new_input, result)
        }
    )
}


fn parse_length<'a>(input: &'a[u8], buffer: &mut [u8]) -> IResult<&'a[u8], (), ()>
{
    let (input, bytes) = take_bytes(buffer.len())(input)?;
    buffer.copy_from_slice(bytes);
    
    Ok((input, ()))
}


pub fn uint(input: &[u8], length: usize) -> IResult<&[u8], u64, ()>
{
    assert!(length <= size_of::<u64>(), format!(
        "invalid length for uint (expected n<{:?}, found {:?})",
        size_of::<u64>(), length,
    ));

    let mut buffer = [0u8; size_of::<u64>()];
    let (input, _) = parse_length(input, &mut buffer[(size_of::<u64>()-length)..])?;

    Ok((input, u64::from_be_bytes(buffer)))
}


pub fn int(input: &[u8], length: usize) -> IResult<&[u8], i64, ()>
{
    assert!(length <= size_of::<i64>(), format!(
        "invalid length for int (expected n<{:?}, found {:?})",
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
    assert!(length == size_of::<f32>(), format!(
        "invalid length for f32 (expected {:?}, found {:?})",
        size_of::<f32>(), length,
    ));

    let mut buffer = [0u8; size_of::<f32>()];
    let (input, _) = parse_length(input, &mut buffer)?;

    Ok((input, f32::from_be_bytes(buffer)))
}


pub fn float64(input: &[u8], length: usize) -> IResult<&[u8], f64, ()>
{
    assert!(length == size_of::<f64>(), format!(
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
    let (input, bytes) = take_bytes(length)(input)?;
    let result = std::str::from_utf8(bytes).or(Err(nom::Err::Failure(())))?;

    Ok((input, result))
}


pub fn date(input: &[u8], length: usize) -> IResult<&[u8], i64, ()>
{
    assert!(length == size_of::<i64>(), format!(
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
    use rstest::*;

    #[rstest(source, bit_offset, expt_result,
        case(
            &[0b_0100_1010, 0b_1010_0101], 3,
            ((&[0b_1010_0101][..], 0), (0b0_1010_u8, 5)),
        ),
        case(
            &[0b_0100_1010, 0b_1010_0101], 0,
            ((&[0b_0100_1010, 0b_1010_0101][..], 0), (0u8, 0)),
        ),
    )]
    fn test_take_rem(
        source: &'static [u8],
        bit_offset: usize,
        expt_result: ((&'static [u8], usize), (u8, usize)),
    ) {
        assert_eq!(
            take_rem::<_, ()>()((source, bit_offset)),
            Ok(expt_result),
        );
    }

    #[rstest(source, bit_offset, max_count, expt_result,
        case(
            &[0b_0000_0000, 0b_0100_1010], 3, usize::MAX,
            ((&[0b_0100_1010][..], 1), 6),
        ),
        case(
            &[0b_1110_0000, 0b_0100_1010], 3, usize::MAX,
            ((&[0b_0100_1010][..], 1), 6),
        ),
        case(
            &[0b_0000_0000, 0b_0100_1010], 3, 5,
            ((&[0b_0100_1010][..], 0), 5),
        ),
    )]
    fn test_take_zeros(
        source: &'static [u8],
        bit_offset: usize,
        max_count: usize,
        expt_result: ((&'static [u8], usize), usize),
    ) {
        assert_eq!(
            take_zeros::<_, _, ()>(max_count)((source, bit_offset)),
            Ok(expt_result),
        );
    }


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
            date(&source[..], 8),
            Ok((&source[8..], i64::from_be_bytes(
                [0x40, 0x01, 0xFF, 0x00, 0x40, 0x01, 0xFF, 0x00],
            ))),
        );
    }
}
