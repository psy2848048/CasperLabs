use types::U512;

pub fn sqrt_for_u512(number: U512) -> U512 {
    if number == U512::zero() || number == U512::one() {
        return number;
    }

    let mut start = U512::one();
    let mut end = number.clone();
    let mut res = U512::zero();

    while start <= end {
        let mid = (start + end) / 2;

        if mid * mid == number {
            return mid;
        }

        if mid * mid < number {
            start = mid + 1;
            res = mid;
        } else {
            end = mid - 1;
        }
    }

    res
}