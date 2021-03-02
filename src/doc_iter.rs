use crate::schema_types::*;

pub trait ElementStream {
    type Value;
    fn read(&self, input: &[u8]) -> Self::Value;
    fn overwrite(&self, output: &mut [u8], value: Self::Value);
}

impl<T: UIntElement + ?IntElement + ?FloatElement + ?DateElement> ElementStream for T {
    type Value = u64;
    fn read(&self, input: &[u8]) -> Self::Value {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: Self::Value) {
        todo!()
    }
}

impl<T: IntElement> ElementStream for T {
    type Value = i64;
    fn read(&self, input: &[u8]) -> Self::Value {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: Self::Value) {
        todo!()
    }
}

impl<T: FloatElement> ElementStream for T {
    type Value = f64;
    fn read(&self, input: &[u8]) -> Self::Value {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: Self::Value) {
        todo!()
    }
}

impl<T: DateElement> ElementStream for T {
    type Value = i64;
    fn read(&self, input: &[u8]) -> Self::Value {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: Self::Value) {
        todo!()
    }
}
