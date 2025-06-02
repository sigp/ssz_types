use typenum::Unsigned;

// On x64, all typenums always fit usize
#[cfg(target_pointer_width = "64")]
pub fn to_usize<N: Unsigned>() -> usize {
    N::to_usize()
}

// On x32, typenums larger starting from 2**32 do not fit usize,
#[cfg(not(target_pointer_width = "64"))]
pub fn to_usize<N: Unsigned>() -> usize {
    let as_usize = N::to_usize();
    let as_u64 = N::to_u64();
    // If usize == u64 representation - N still fit usize, so
    // no overflow happened
    if as_usize as u64 == as_u64 {
        return as_usize;
    }
    // else we have a choice:
    // Option 1. Loudly panic with as informative message as possible
    #[cfg(not(feature = "cap-typenum-to-usize-overflow"))]
    panic!(
        "Overflow converting typenum U{} to usize (usize::MAX={})",
        as_u64,
        usize::MAX
    );
    // Option 2. Use usize::MAX - this allows working with VariableLists "virtually larger" than the
    // usize, provided the actual number of elements do not exceed usize.
    //
    // One example is Ethereum BeaconChain.validators field that is a VariableList<..., 2**40>,
    // but actual number of validators is far less than 2**32.
    //
    // This option still seems sound, since if the number of elements
    // actually surpass usize::MAX, the machine running this will OOM/segfault/otherwise violently
    // crash the program running this, which is nearly equivalent to panic.
    //
    // Still, the is a double-edged sword, only apply if you can guarantee that none of the
    // VariableList used in your program will have more than usize::MAX elements on the
    // architecture with the smallest usize it will be even run.
    #[cfg(feature = "cap-typenum-to-usize-overflow")]
    usize::MAX
}
