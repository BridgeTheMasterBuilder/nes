use crate::util::bit::Bit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ShiftRegister<T: Default, const N: u8> {
    pub data: T,
}

impl<T: Default, const N: u8> ShiftRegister<T, N> {
    const CAPACITY: u8 = N;

    pub fn new() -> Self {
        debug_assert!((Self::CAPACITY as usize) <= std::mem::size_of::<T>() * 8);

        Self { data: T::default() }
    }
}

macro_rules! generate_shift_implementation {
    ($type:ty) => {
        impl<const N: u8> ShiftRegister<$type, N> {
            pub fn clear(&mut self) {
                self.data = <$type>::default();
            }

            pub fn load(&mut self, data: $type) {
                self.data = data;
            }

            pub fn peek(&self) -> u8 {
                self.data.bit(0) as u8
            }

            pub fn peek_n(&self, n: u8) -> $type {
                self.data.bits_abs(0, n as usize) as $type
            }

            pub fn pop(&mut self) -> u8 {
                let bit = self.data.bit(1) as u8;

                self.data >>= 1;

                bit
            }

            pub fn push(&mut self, bit: u8) {
                self.data = (self.data >> 1) | ((bit as $type) << (Self::CAPACITY - 1));
            }

            pub fn push_n(&mut self, data: u8, n: u8) {
                self.data <<= (Self::CAPACITY - n);

                for i in (0..n).rev() {
                    let bit = data.bit(i as usize) as u8;

                    self.push(bit);
                }
            }

            pub fn take(&mut self) -> $type {
                let data = self.data;

                self.data = <$type>::default();

                data
            }
        }
    };
}

generate_shift_implementation!(u8);
generate_shift_implementation!(u16);
