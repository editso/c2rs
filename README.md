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
c2rs = "0.1.2"

```

# Example
```
fn test(){
    use c2rs::c2rs_def;

    type DWORD = u32;
    const SIZE: usize = 10;

    c2rs_def!(
        struct A{
            DWORD var1;
            DWORD var2;
            union {
                DWORD var4;
                DWORD var5;   
            }var3;
            
            struct {
                u8 var7;
            }var6;

            DWORD array[SIZE];
        };
        
        struct B{
            u8 var1;
        };
        
        // ....
    );
    
    let mut buffer = [1u8; 1024];
    
    unsafe{
        let mut buf = A::from_mut_bytes(buffer.as_mut_ptr());
        let buf = buf.as_mut().unwrap();
        buf.var1 = 10;
        
        assert_eq!(10, buf.var1);
        assert_eq!(10, buffer[0]);
        
        let mut b = B::from_mut_bytes(buffer.as_mut_ptr()).as_mut().unwrap();
        
        assert_eq!(10, b.var1);
    
    }
}
```
