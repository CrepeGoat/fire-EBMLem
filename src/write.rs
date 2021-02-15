use std::cmp::{max, min};
use std::mem::size_of;

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

fn vlen_int(
    output: &mut [u8],
    value: u64,
    min_length: Option<usize>,
    max_length: Option<usize>,
) -> IResult<&mut [u8], usize, ()> {
    //let bitlen = 8*size_of::<u64>() - (value.leading_zeros() as usize);
    let bitlen = 8*size_of::<u64>() - value.leading_zeros() as usize;
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
    let ((output, bit_offset), _) = give_bits(
        (output, bit_offset),
        (source[byte_offset], 8 - bit_offset),
    )?;
    assert_eq!(bit_offset, 0); // -> safe to operate on the byte-level
    let (output, _) = give_bytes(output, &source[byte_offset+1..])?;

    Ok((output, vint_len))
}

fn element_id(output: &mut [u8], value: u64) -> IResult<&mut [u8], usize, ()> {
    if value == 0 {
        return Err(nom::Err::Error(()));
    }

    vlen_int(
        output,
        value,
        Some(((value.count_ones() / 7) + 1) as usize), // ensures that VINT_DATA of id's are not all 1's
        Some(4),
    )
}

fn element_len(
    output: &mut [u8],
    value: u64,
    bytelen: Option<usize>,
) -> IResult<&mut [u8], usize, ()> {
    vlen_int(output, value, bytelen, Some(8))
}


fn uint(output: &mut [u8], value: u64, length: usize) -> IResult<&mut [u8], (), ()> {
    let byte_offset = size_of::<u64>().checked_sub(length).ok_or(nom::Err::Error(()))?;
    if 8 * byte_offset > (value.leading_zeros() as usize) {
        return Err(nom::Err::Error(()));
    }

    let source = value.to_be_bytes();
    give_bytes(output, &source[byte_offset..])
}

fn int(output: &mut [u8], value: i64, length: usize) -> IResult<&mut [u8], (), ()> {
    let byte_offset = size_of::<u64>().checked_sub(length).ok_or(nom::Err::Error(()))?;
    let value_spare_bits = max(value.leading_zeros(), value.leading_ones()) - 1; // need leading bit for sign
    if 8 * byte_offset > (value_spare_bits as usize) {
        return Err(nom::Err::Error(()));
    }

    let source = value.to_be_bytes();
    give_bytes(output, &source[byte_offset..])
}

fn float32(output: &mut [u8], value: f32, length: usize) -> IResult<&mut [u8], (), ()> {
    if length != size_of::<f32>() {
        return Err(nom::Err::Error(()));
    }
    let source = value.to_be_bytes();
    give_bytes(output, &source[..])
}

fn float64(output: &mut [u8], value: f64, length: usize) -> IResult<&mut [u8], (), ()> {
    if length != size_of::<f64>() {
        return Err(nom::Err::Error(()));
    }
    let source = value.to_be_bytes();
    give_bytes(output, &source[..])
}

fn string<'a>(output: &'a mut [u8], value: &str) -> IResult<&'a mut [u8], (), ()> {
    give_bytes(output, value.as_bytes())
}

fn date<'a>(output: &'a mut [u8], value: i64, length: usize) -> IResult<&'a mut [u8], (), ()> {
    if length != size_of::<i64>() {
        return Err(nom::Err::Error(()));
    }
    int(output, value, length)
}

fn binary<'a>(output: &'a mut [u8], value: &[u8]) -> IResult<&'a mut [u8], (), ()> {
    give_bytes(output, value)
}
