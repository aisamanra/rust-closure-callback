#![feature(libc)]
extern crate libc;

use libc::*;
use std::mem;

extern {
    // Our external C function here takes two arguments: one is a
    // function pointer which takes an int and a void pointer, and the
    // other is the data we pass as the void pointer.
    fn call_callback(cb: extern fn(i32, *const c_void) -> (),
                     data: *const c_void);
}

// This function is what we're actually going to provide as the
// function pointer to call_callback: it takes a number and a void
// pointer...
extern "C" fn apply_closure(n: i32, data: *const c_void) {
    // and here it interprets the void pointer as a reference to a
    // pointer to a function. The two layers are doing something
    // important: pointers to traits in Rust are "fat pointers",
    // i.e. they are actually a function-pointer/data-pointer pair. So
    // what we're doing is getting a pointer to those two pointer
    // pairs, and then using those to call the code with the
    // appropriate data.
    let closure: &mut Box<Fn(i32) -> ()> = unsafe { mem::transmute(data) };
    // And then we can just call the closure!
    closure(n);
}

// This is a nicer wrapper function to providing the closure/callback
// to C:
fn callback<F>(cb: F)
    where F: Fn(i32)
{
    // For the reasons articulated above, we create a pointer to a
    // (fat) pointer to the closure
    let boxed_cb: Box<Box<Fn(i32)>> = Box::new(Box::new(cb));
    // and then we call the C function with our 'apply_closure'
    // function as well as the actual closure:
    unsafe {
        call_callback(apply_closure,
                      Box::into_raw(boxed_cb) as *const c_void);
    }
}

// And now we can use it!
fn main() {
    let x = 4;
    callback(|n| println!("Got: {}", n + x));
}
