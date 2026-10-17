# rust-num

Import numbers by their English names.

```rust
use rust_num::numbers;

#[numbers]
use rust_num::{twothousandtwentysix, minus_forty_two, three_point_one_four};

fn main() {
    let year: u32 = twothousandtwentysix();
    let cold: i64 = minus_forty_two();
    let pi: f64 = three_point_one_four();

    println!("{year} {cold} {pi}"); // 2026 -42 3.14
}
```

Run the demo with `cargo run`.

## Names

- Underscores are optional. `forty_two` and `fortytwo` are the same name.
- Words run from `zero` to `undecillion`. The largest value is the limit of `u128`.
- `and` is allowed: `one_hundred_and_five`.
- `minus` or `negative` at the start makes the value negative.
- `point` starts a decimal. Each word after it is one digit: `zero_point_zero_five` is `0.05`.

The macro rejects names that are not well-formed numbers, for example `onetwo`, `thousand_one` and `twenty_ten`. It also rejects `twenty_six_hundred`, because `hundred` follows a single digit only.

## Other forms

```rust
#[numbers]
use rust_num::{five as v, seven};     // rename

#[numbers]
pub use rust_num::{twelve};           // the functions are pub
```

A `use` with an attribute also works inside a function body.
