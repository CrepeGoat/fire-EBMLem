pub mod parse {
    use std::cmp::min;
    use std::mem::size_of;
    use std::ops::RangeFrom;

    use nom::{
        bits::streaming::take as take_bits, bytes::streaming::take as take_bytes,
        error::ParseError, Err, IResult, InputIter, InputLength, Needed, Slice, ToUsize,
    };

    fn take_rem<I, E: ParseError<(I, usize)>>(
    ) -> impl Fn((I, usize)) -> IResult<(I, usize), (u8, usize), E>
    where
        I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    {
        move |(input, bit_offset): (I, usize)| {
            let bitlen = (8usize - bit_offset) % 8usize;
            take_bits(bitlen)((input, bit_offset))
                .map(move |((input, bit_offset), bits)| ((input, bit_offset), (bits, bitlen)))
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
            let mut item = input
                .iter_elements()
                .next()
                .ok_or_else(|| Err::Incomplete(Needed::new(1)))?;
            item &= 0xFF >> bit_offset; // mask out first `bit_offset` bits

            streak_len += (item.leading_zeros() as usize) - bit_offset;
            while item.leading_zeros() == 8 && streak_len <= max_count {
                input = input.slice(1..);
                if streak_len == max_count {
                    break;
                };
                item = input
                    .iter_elements()
                    .next()
                    .ok_or_else(|| Err::Incomplete(Needed::new(1)))?;
                streak_len += item.leading_zeros() as usize;
            }
            streak_len = min(streak_len, max_count);

            Ok(((input, (streak_len + bit_offset) % 8), streak_len))
        }
    }

    macro_rules! make_vlen_parser {
        ($func_name:ident, $uint:ty) => {
            fn $func_name(input: &[u8]) -> IResult<&[u8], ($uint, usize), ()> {
                // Parse length from stream
                let ((input, bit_offset), len) = take_zeros(size_of::<$uint>())((input, 0))?;
                if len >= size_of::<$uint>() {
                    return Err(nom::Err::Error(()));
                }
                let ((input, bit_offset), _) =
                    take_bits::<_, usize, _, ()>(1u8)((input, bit_offset))?;
                let ((input, _), (leftover_bits, _)) = take_rem()((input, bit_offset))?;
                let (input, bytes) = take_bytes(len)(input)?;

                let mut buffer = [0u8; size_of::<$uint>()];
                buffer[size_of::<$uint>() - len - 1] = leftover_bits;
                buffer[(size_of::<$uint>() - len)..].copy_from_slice(bytes);

                Ok((input, (<$uint>::from_be_bytes(buffer), len + 1)))
            }
        };
    }

    make_vlen_parser!(vlen_to_u32, u32);
    make_vlen_parser!(vlen_to_u64, u64);

    const RESERVED_ELEMENT_ID: u32 = u32::MAX;

    pub fn element_id(input: &[u8]) -> IResult<&[u8], u32, ()> {
        let (new_input, (result, bytelen)) = vlen_to_u32(input)?;

        if result == 0 || result.count_ones() == 7 * (bytelen as u32) {
            // if all non-length bits are 0's or 1's
            // corner-case: reserved ID's
            return Ok((new_input, RESERVED_ELEMENT_ID));
        }
        let sig_bits = 8 * size_of::<u32>() - ((result + 1).leading_zeros() as usize);
        if sig_bits <= 7 * (bytelen - 1) {
            // element ID's must use the smallest representation possible
            return Err(nom::Err::Error(()));
        }

        Ok((new_input, result))
    }

    const UNKNOWN_ELEMENT_LEN: u64 = u64::MAX;

    pub fn element_len(input: &[u8]) -> IResult<&[u8], u64, ()> {
        let (new_input, (result, bytelen)) = vlen_to_u64(input)?;

        Ok(if result.count_ones() == 7 * (bytelen as u32) {
            // if all non-length bits are 1's
            // corner-case: reserved ID's
            (new_input, UNKNOWN_ELEMENT_LEN)
        } else {
            (new_input, result)
        })
    }

    fn parse_length<'a>(input: &'a [u8], buffer: &mut [u8]) -> IResult<&'a [u8], (), ()> {
        let (input, bytes) = take_bytes(buffer.len())(input)?;
        buffer.copy_from_slice(bytes);

        Ok((input, ()))
    }

    pub fn uint(input: &[u8], length: usize) -> IResult<&[u8], u64, ()> {
        assert!(
            length <= size_of::<u64>(),
            format!(
                "invalid length for uint (expected n<{:?}, found {:?})",
                size_of::<u64>(),
                length,
            )
        );

        let mut buffer = [0u8; size_of::<u64>()];
        let i0 = size_of::<i64>() - length;
        let (input, _) = parse_length(input, &mut buffer[i0..])?;

        Ok((input, u64::from_be_bytes(buffer)))
    }

    pub fn int(input: &[u8], length: usize) -> IResult<&[u8], i64, ()> {
        assert!(
            length <= size_of::<i64>(),
            format!(
                "invalid length for int (expected n<{:?}, found {:?})",
                size_of::<i64>(),
                length,
            )
        );

        let buffer_fill: u8 = match take_bits(1usize)((input, 0))? {
            ((_, 1), 0) => 0x00,
            ((_, 1), 1) => 0xFF,
            _ => unreachable!(),
        };
        let mut buffer = [buffer_fill; size_of::<i64>()];
        let i0 = size_of::<i64>() - length;
        let (input, _) = parse_length(input, &mut buffer[i0..])?;

        Ok((input, i64::from_be_bytes(buffer)))
    }

    pub fn float32(input: &[u8], length: usize) -> IResult<&[u8], f32, ()> {
        assert!(
            length == size_of::<f32>(),
            format!(
                "invalid length for f32 (expected {:?}, found {:?})",
                size_of::<f32>(),
                length,
            )
        );

        let mut buffer = [0u8; size_of::<f32>()];
        let (input, _) = parse_length(input, &mut buffer)?;

        Ok((input, f32::from_be_bytes(buffer)))
    }

    pub fn float64(input: &[u8], length: usize) -> IResult<&[u8], f64, ()> {
        assert!(
            length == size_of::<f64>(),
            format!(
                "invalid length for f64 (expected {:?}, found {:?})",
                size_of::<f64>(),
                length,
            )
        );

        let mut buffer = [0u8; size_of::<f64>()];
        let (input, _) = parse_length(input, &mut buffer)?;

        Ok((input, f64::from_be_bytes(buffer)))
    }

    pub fn ascii_str(input: &[u8], length: usize) -> IResult<&[u8], &str, ()> {
        let (input, result) = unicode_str(input, length)?;
        if !result[..].is_ascii() {
            return Err(nom::Err::Error(()));
        }

        Ok((input, result))
    }

    pub fn unicode_str(input: &[u8], length: usize) -> IResult<&[u8], &str, ()> {
        let (input, bytes) = take_bytes(length)(input)?;
        let result = std::str::from_utf8(bytes).or(Err(nom::Err::Error(())))?;

        Ok((input, result))
    }

    pub fn date(input: &[u8], length: usize) -> IResult<&[u8], i64, ()> {
        assert!(
            length == size_of::<i64>(),
            format!(
                "invalid length for timestamp (expected {:?}, found {:?})",
                size_of::<i64>(),
                length,
            )
        );

        int(input, length)
    }

    pub fn binary(input: &[u8], length: usize) -> IResult<&[u8], &[u8], ()> {
        take_bytes(length)(input)
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
            assert_eq!(take_rem::<_, ()>()((source, bit_offset)), Ok(expt_result),);
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
            let source = [0x40, 0x7F, 0xFF];
            assert_eq!(element_id(&source[..]), Ok((&source[2..], 0x7F)));
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
            assert_eq!(
                int(&source[..], 1),
                Ok((&source[1..], i8::from_be_bytes([source[0]]) as i64))
            );
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
            let s = "知ら ない の か ？ 死神 の 霊 絡 は 色 が 違う って こと ｡";
            let source = s.as_bytes();
            assert_eq!(
                unicode_str(&source[..], 25),
                Ok((&source[25..], "知ら ない の か ？"))
            );
        }

        #[test]
        fn test_date() {
            let source = [0x40, 0x01, 0xFF, 0x00, 0x40, 0x01, 0xFF, 0x00, 0xFF, 0xFF];
            assert_eq!(
                date(&source[..], 8),
                Ok((
                    &source[8..],
                    i64::from_be_bytes([0x40, 0x01, 0xFF, 0x00, 0x40, 0x01, 0xFF, 0x00],)
                )),
            );
        }
    }
}

pub mod serialize {
    use std::cmp::{max, min, Ordering};
    use std::mem::size_of;
    use std::num::NonZeroU32;

    use nom::{Err, IResult, Needed};

    fn give_bits(
        (output, bit_offset): (&mut [u8], usize),
        (source, length): (u8, usize),
    ) -> IResult<(&mut [u8], usize), (), ()> {
        if length == 0 {
            return Ok(((output, bit_offset), ()));
        }

        let size_rem = 8 - bit_offset;
        let right_offset = size_rem.checked_sub(length).ok_or(nom::Err::Error(()))?;

        let bitmask = (!(0xFFu8 >> length)) >> bit_offset;
        output[0] = (output[0] & !bitmask) | ((source << right_offset) & bitmask);

        Ok(if right_offset == 0 {
            ((&mut output[1..], 0), ())
        } else {
            ((output, bit_offset + length), ())
        })
    }

    fn give_bytes<'a>(output: &'a mut [u8], source: &[u8]) -> IResult<&'a mut [u8], (), ()> {
        if output.len() < source.len() {
            return Err(Err::Incomplete(Needed::new(source.len() - output.len())));
        }
        output[..source.len()].copy_from_slice(source);

        Ok((&mut output[source.len()..], ()))
    }

    fn skip_bytes(output: &mut [u8], length: usize) -> IResult<&mut [u8], (), ()> {
        if output.len() < length {
            return Err(Err::Incomplete(Needed::new(length - output.len())));
        }

        Ok((&mut output[length..], ()))
    }

    fn vlen_int(
        output: &mut [u8],
        value: u64,
        min_length: Option<usize>,
        max_length: Option<usize>,
    ) -> IResult<&mut [u8], usize, ()> {
        let bitlen = 8 * size_of::<u64>() - value.leading_zeros() as usize;
        let mut vint_len = bitlen.saturating_sub(1) / 7 + 1;

        if let Some(length) = min_length {
            if vint_len < length {
                vint_len = length;
            }
        }
        let length = max_length.map_or(8, |x| min(x, 8));
        if vint_len > length {
            return Err(nom::Err::Error(()));
        }

        let bit_offset = 0;
        let ((output, bit_offset), _) = give_bits((output, bit_offset), (0, vint_len - 1))?;
        let ((output, bit_offset), _) = give_bits((output, bit_offset), (1, 1))?;

        let source = value.to_be_bytes();
        let byte_offset = size_of::<u64>() - vint_len;
        let ((output, bit_offset), _) =
            give_bits((output, bit_offset), (source[byte_offset], 8 - bit_offset))?;
        assert_eq!(bit_offset, 0); // -> safe to operate on the byte-level
        let (output, _) = give_bytes(output, &source[byte_offset + 1..])?;

        Ok((output, vint_len))
    }

    pub fn element_id(output: &mut [u8], value: NonZeroU32) -> IResult<&mut [u8], usize, ()> {
        let value = value.get();
        vlen_int(
            output,
            value.into(),
            Some(((value.count_ones() / 7) + 1) as usize), // ensures that VINT_DATA of id's are not all 1's
            Some(4),
        )
    }

    pub fn element_len(
        output: &mut [u8],
        value: u64,
        bytelen: Option<usize>,
    ) -> IResult<&mut [u8], usize, ()> {
        vlen_int(output, value, bytelen, Some(8))
    }

    pub fn uint(output: &mut [u8], value: u64, length: usize) -> IResult<&mut [u8], (), ()> {
        let byte_offset = size_of::<u64>()
            .checked_sub(length)
            .ok_or(nom::Err::Error(()))?;
        if 8 * byte_offset > (value.leading_zeros() as usize) {
            return Err(nom::Err::Error(()));
        }

        let source = value.to_be_bytes();
        give_bytes(output, &source[byte_offset..])
    }

    pub fn int(output: &mut [u8], value: i64, length: usize) -> IResult<&mut [u8], (), ()> {
        let byte_offset = size_of::<u64>()
            .checked_sub(length)
            .ok_or(nom::Err::Error(()))?;
        let value_spare_bits = max(value.leading_zeros(), value.leading_ones()) - 1; // need leading bit for sign
        if 8 * byte_offset > (value_spare_bits as usize) {
            return Err(nom::Err::Error(()));
        }

        let source = value.to_be_bytes();
        give_bytes(output, &source[byte_offset..])
    }

    pub fn float32(output: &mut [u8], value: f32, length: usize) -> IResult<&mut [u8], (), ()> {
        if length != size_of::<f32>() {
            return Err(nom::Err::Error(()));
        }
        let source = value.to_be_bytes();
        give_bytes(output, &source[..])
    }

    pub fn float64(output: &mut [u8], value: f64, length: usize) -> IResult<&mut [u8], (), ()> {
        if length != size_of::<f64>() {
            return Err(nom::Err::Error(()));
        }
        let source = value.to_be_bytes();
        give_bytes(output, &source[..])
    }

    pub fn string<'a>(
        output: &'a mut [u8],
        value: &str,
        length: usize,
    ) -> IResult<&'a mut [u8], (), ()> {
        let value = value.as_bytes();
        match length.cmp(&value.len()) {
            Ordering::Less => Err(nom::Err::Error(())),
            Ordering::Equal => give_bytes(output, value),
            Ordering::Greater => {
                let (output, _) = give_bytes(output, value)?;
                let (output, _) = give_bytes(output, b"\0")?; // null-terminate the string
                skip_bytes(output, length - (value.len() + 1))
            }
        }
    }

    pub fn date(output: &mut [u8], value: i64, length: usize) -> IResult<&mut [u8], (), ()> {
        if length != size_of::<i64>() {
            return Err(nom::Err::Error(()));
        }
        int(output, value, length)
    }

    pub fn binary<'a>(output: &'a mut [u8], value: &[u8]) -> IResult<&'a mut [u8], (), ()> {
        give_bytes(output, value)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rstest::*;

        #[rstest(output, bit_offset, source, bitlen, expt_output,
            case([0x00, 0x00], 4, 0xFF, 2, &[0x0C, 0x00]),
        )]
        fn test_give_bits(
            mut output: [u8; 2],
            bit_offset: usize,
            source: u8,
            bitlen: usize,
            expt_output: &[u8],
        ) {
            let result = give_bits((&mut output, bit_offset), (source, bitlen));
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(output, source, expt_output,
            case([0x00, 0x00], &[0xFF][..], &[0xFF, 0x00]),
        )]
        fn test_give_bytes(mut output: [u8; 2], source: &'static [u8], expt_output: &[u8]) {
            let result = give_bytes(&mut output, source);
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(value, expt_output,
            case(0x01, &[0x81, 0x00, 0x00, 0x00, 0x00]),
            case(0x2345, &[0x63, 0x45, 0x00, 0x00, 0x00]),
            case(0x7F, &[0x40, 0x7F, 0x00, 0x00, 0x00]),
        )]
        fn test_element_id(value: u32, expt_output: &[u8]) {
            let mut output = [0x00u8; 5];
            let result = element_id(&mut output[..], NonZeroU32::new(value).unwrap());
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(value, length, expt_output,
            case(0x2345, None, &[0x63, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            case(0x7F, None, &[0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            case(0x7F, Some(2), &[0x40, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        )]
        fn test_element_len(value: u64, length: Option<usize>, expt_output: &[u8]) {
            let mut output = [0x00u8; 9];
            let result = element_len(&mut output[..], value, length);
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(value, length, expt_output,
            case(0x01, 1, &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            case(0x01, 2, &[0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        )]
        fn test_uint(value: u64, length: usize, expt_output: &[u8]) {
            let mut output = [0x00u8; 9];
            let result = uint(&mut output[..], value, length);
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(value, length, expt_output,
            case(-1, 1, &[0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            case(-1, 2, &[0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        )]
        fn test_int(value: i64, length: usize, expt_output: &[u8]) {
            let mut output = [0x00u8; 9];
            let result = int(&mut output[..], value, length);
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(value, length, expt_output,
            case(1.0, 4, &[0x3F, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        )]
        fn test_float32(value: f32, length: usize, expt_output: &[u8]) {
            let mut output = [0x00u8; 9];
            let result = float32(&mut output[..], value, length);
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(value, length, expt_output,
            case(1.0, 8, &[0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        )]
        fn test_float64(value: f64, length: usize, expt_output: &[u8]) {
            let mut output = [0x00u8; 9];
            let result = float64(&mut output[..], value, length);
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(value, expt_output,
            case(&"hello", &[0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x00, 0xFF, 0xFF, 0xFF]),
            case(&"え？", &[0xE3, 0x81, 0x88, 0xEF, 0xBC, 0x9F, 0xFF, 0xFF, 0xFF]),
        )]
        fn test_string(value: &str, expt_output: &[u8]) {
            let mut output = [0xFFu8; 9];
            let result = string(&mut output[..], value, 6);
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }

        #[rstest(value, length, expt_output,
            case(-1, 8, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]),
        )]
        fn test_date(value: i64, length: usize, expt_output: &[u8]) {
            let mut output = [0x00u8; 9];
            let result = date(&mut output[..], value, length);
            assert!(result.is_ok());
            assert_eq!(output, expt_output);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::num::NonZeroU32;

    proptest! {
        #[test]
        fn write_read_eq_element_id(value in 1u32..((u32::MAX >> 4)-1)) {
            let mut buffer = [0x00u8; 5];

            let (_output, _bytelen) = serialize::element_id(
                &mut buffer[..],
                NonZeroU32::new(value).expect("`NonZeroU32::new` failed"),
            ).expect("failed to write value");
            let (_input, result) = parse::element_id(&buffer[..]).expect("failed to read value");

            prop_assert_eq!(result, value);
        }

        #[test]
        fn write_read_eq_element_len(value in 0u64..((u64::MAX >> 8)-1)) {
            let mut buffer = [0x00u8; 9];

            let (_output, _bytelen) = serialize::element_len(&mut buffer[..], value, None).expect("failed to write value");
            let (_input, result) = parse::element_len(&buffer[..]).expect("failed to read value");

            prop_assert_eq!(result, value);
        }

        #[test]
        fn write_read_eq_uint(value: u64) {
            let mut buffer = [0x00u8; 9];

            let (_output, _bytelen) = serialize::uint(&mut buffer[..], value, 8).expect("failed to write value");
            let (_input, result) = parse::uint(&buffer[..], 8).expect("failed to read value");

            prop_assert_eq!(result, value);
        }

        #[test]
        fn write_read_eq_int(value: i64) {
            let mut buffer = [0x00u8; 9];

            let (_output, _bytelen) = serialize::int(&mut buffer[..], value, 8).expect("failed to write value");
            let (_input, result) = parse::int(&buffer[..], 8).expect("failed to read value");

            prop_assert_eq!(result, value);
        }

        #[test]
        #[allow(clippy::float_cmp)]
        fn write_read_eq_float32(value: f32) {
            let mut buffer = [0x00u8; 9];

            let (_output, _bytelen) = serialize::float32(&mut buffer[..], value, 4).expect("failed to write value");
            let (_input, result) = parse::float32(&buffer[..], 4).expect("failed to read value");

            prop_assert_eq!(result, value);
        }

        #[test]
        #[allow(clippy::float_cmp)]
        fn write_read_eq_float64(value: f64) {
            let mut buffer = [0x00u8; 9];

            let (_output, _bytelen) = serialize::float64(&mut buffer[..], value, 8).expect("failed to write value");
            let (_input, result) = parse::float64(&buffer[..], 8).expect("failed to read value");

            prop_assert_eq!(result, value);
        }

        #[test]
        fn write_read_eq_date(value: i64) {
            let mut buffer = [0x00u8; 9];

            let (_output, _bytelen) = serialize::date(&mut buffer[..], value, 8).expect("failed to write value");
            let (_input, result) = parse::date(&buffer[..], 8).expect("failed to read value");

            prop_assert_eq!(result, value);
        }

        #[test]
        fn write_read_eq_ascii(value in "[ -~]{0,8}") {
            let mut buffer = [0xFFu8; 9];

            let (_output, _bytelen) = serialize::string(&mut buffer[..], &value, 8).expect("failed to write value");
            let (_input, result) = parse::ascii_str(&buffer[..], 8).expect("failed to read value");

            prop_assert_eq!(result, value);
        }

        //#[test] fn write_read_eq_unicode

    }
}
