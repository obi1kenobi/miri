// We want to control preemption here. Stacked borrows interferes by having its own accesses.
//@compile-flags: -Zmiri-preemption-rate=0 -Zmiri-disable-stacked-borrows

use std::thread::spawn;

#[derive(Copy, Clone)]
struct EvilSend<T>(pub T);

unsafe impl<T> Send for EvilSend<T> {}
unsafe impl<T> Sync for EvilSend<T> {}

extern "Rust" {
    fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize);
}

pub fn main() {
    // Shared atomic pointer
    let pointer: *mut usize = Box::into_raw(Box::new(0usize));
    let ptr = EvilSend(pointer);

    unsafe {
        let j1 = spawn(move || {
            let _val = *ptr.0;
        });

        let j2 = spawn(move || {
            __rust_dealloc(
                //~^ ERROR: Data race detected between (1) Read on thread `<unnamed>` and (2) Deallocate on thread `<unnamed>`
                ptr.0 as *mut _,
                std::mem::size_of::<usize>(),
                std::mem::align_of::<usize>(),
            );
        });

        j1.join().unwrap();
        j2.join().unwrap();
    }
}
