# c2rs 
This is a macro that converts the `struct` of the `c` language into a `rust struct`

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/c2rs.svg
[crates-url]: https://crates.io/crates/c2rs
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/editso/c2rs/blob/master/LICENSE

# use
```
// Cargo.toml

[dependencies]
c2rs = "0.1.1"

```

# example
```
c2rs_def!(
    struct A{

    };

    struct B{
        
    };

    // ...
);

```
