pub fn _generate_random_u64(random_state: &mut u32) -> u64 {
    // `& 0xFFFF` operation cuts off first 16 most significant bits from 32 bit integer
    _xor_shift_mutate(random_state);
    let random_u64_1 = (*random_state & 0xFFFF) as u64;
    _xor_shift_mutate(random_state);
    let random_u64_2 = (*random_state & 0xFFFF) as u64;
    _xor_shift_mutate(random_state);
    let random_u64_3 = (*random_state & 0xFFFF) as u64;
    _xor_shift_mutate(random_state);
    let random_u64_4 = (*random_state & 0xFFFF) as u64;

    random_u64_1 | (random_u64_2 << 16) | (random_u64_3 << 32) | (random_u64_4 << 48)
}

fn _xor_shift_mutate(random_state: &mut u32) {
    *random_state ^= *random_state << 13;
    *random_state ^= *random_state >> 17;
    *random_state ^= *random_state << 5;
}
