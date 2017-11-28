// In this case, I'm using 'mut' in a few places it's not strictly
// necessary, as a reminder. In particular: when we pass the 'self'
// object in to our `callback` function, we're doing so because the
// callback will eventually have mutable access to it. But in the
// `callback` function itself, we don't use it mutably: we cast it to
// an opaque C pointer, which means from Rust's point of view, we
// don't _actually_ mutate it. But we're going to cast it back to a
// mutable pointer eventually, which means passing a pointer in to
// `callback` might involve it being mutated, even in Rust doesn't
// realize it!
#![allow(unused_mut)]

#![feature(libc)]
extern crate libc;

use libc::*;
use std::mem;

// Here is a very silly struct with some internal state
#[derive(Debug)]
struct Thingy {
    count: i32,
}

// And a few methods on it that visibly update that state
impl Thingy {
    fn add_count(&mut self, n: i32) {
        self.count += n;
        println!("updated count to {}", self.count);
    }

    fn decrement_count(&mut self, _: i32) {
        self.count -= 1;
        println!("decremented count to {}", self.count);
    }
}

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
    // pair of pointers, one to a function and one to the 'self'
    // object we're concerned with.
    let mut payload: &mut (Box<Fn(&mut Thingy, i32) -> ()>, &mut Thingy) =
        unsafe { mem::transmute(data) };

    // Then, we call the closure with the 'self' object as first
    // parameter. If the function is a method of Thingy, then this
    // will work identically.
    payload.0(&mut payload.1, n);
}

impl Thingy {
    // This is a nicer wrapper function to providing the closure/callback
    // to C:
    fn callback<F>(&mut self, cb: F)
        where F: Fn(&mut Thingy, i32)
    {
        // For the reasons articulated above, we create a pointer to a
        // pair of the boxed function _and_ the pointer to Thingy
        let boxed_cb: Box<(Box<Fn(&mut Thingy, i32)>, &mut Thingy)> =
            Box::new((Box::new(cb), self));
        // and then we call the C function with our 'apply_closure'
        // function as well as the actual closure/self-pointer pair,
        // passed as a void pointer
        unsafe {
            call_callback(apply_closure,
                          Box::into_raw(boxed_cb) as *const c_void);
        }
    }
}

// And now we can use it!
fn main() {
    let mut t = Thingy { count: 0 };
    t.callback(Thingy::add_count);
    t.callback(Thingy::decrement_count);
    println!("Final status: {:?}", t);
}
