# size_of_const_macro
Provides the `SizeOf` derive macro for generating a constant of the type's size.
Not designed to work on types with generics.  
The MSRV is 1.56.

# Example
Derive it on your object declaration.
```
use size_of_const_macro::SizeOf;

#[derive(SizeOf)]
struct FooBar {
	short: u16,
	long: u32,
}

fn main() {
	assert_eq!(core::mem::size_of::<FooBar>(), SIZE_OF_FOO_BAR);
}
```

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `size_of_const_macro` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
