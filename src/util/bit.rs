pub trait Bit {
    type Output;

    fn bit(self, n: usize) -> bool;
    fn bits(self, n: usize, m: usize) -> Self::Output;
    fn bits_abs(self, n: usize, m: usize) -> Self::Output;
}

macro_rules! generate_bit_implementation {
    ($type:ty) => {
        impl Bit for $type {
            type Output = $type;

            fn bit(self, n: usize) -> bool {
                let mask = 1 << n;

                (self & mask) != 0
            }

            fn bits(self, n: usize, m: usize) -> Self::Output {
                let mask_m = (1 << (m + 1)) - 1;
                let mask_n = (1 << n) - 1;
                let mask = mask_m & !mask_n;

                (self & mask as $type) >> n
            }

            fn bits_abs(self, n: usize, m: usize) -> Self::Output {
                let mask_m = (1 << (m + 1)) - 1;
                let mask_n = (1 << n) - 1;
                let mask = mask_m & !mask_n;

                self & mask as $type
            }
        }
    };
}

generate_bit_implementation!(u8);
generate_bit_implementation!(u16);
