# RMPP - Rust MessagePack Precise 📐
RMPP is a pure Rust MessagePack implementation based on [RMP](https://crates.io/crates/rmp) crate. It aims to accurately preserve the original types metadata in **Rust** as well as **JavaScript**. It's incredibly useful in cases the data is sensitive to strict typing.

---

## Rust 🦀
You can install the crate using the following cargo command:
```sh
cargo add rmpp
```

Sample unpack usage:
```rust
use rmpp;

let binary: Vec<u8> = vec![
    0x82, 0xA3, 0x69, 0x6E, 0x74, 0x01, 0xA5,
    0x66, 0x6C, 0x6F, 0x61, 0x74, 0xCB, 0x3F,
    0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];

let json_string: String = rmpp::unpack_json(&binary, Some(true))?;
```

Sample pack usage:
```rust
use rmpp;

let rmpp_json: String = r###"
{
    "raw_marker": 195,
    "basic_type": "Bool",
    "data": {
        "type": "Bool",
        "value": true
    }
}
"###;

let vec: Vec<u8> = rmpp::pack_json(json);
assert_eq!(vec![0xC3], vec);
```

The crate also provides a handy `MsgPackEntry` type that `rmpp::pack()` and `rmpp::unpack()` work with.

---

## JavaScript ⭐
You can install the package using the following npm command:
```sh
npm i rmpp
```

Sample unpack usage:
```ts
import { unpack_json } from 'rmpp';

const binary: Uint8Array = new Uint8Array([
    0x82, 0xA3, 0x69, 0x6E, 0x74, 0x01, 0xA5,
    0x66, 0x6C, 0x6F, 0x61, 0x74, 0xCB, 0x3F,
    0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
]);

const jsonString: string = unpack_json(binary);
```

Sample pack usage:
```ts
import { pack_json } from 'rmpp';

const rmppJson: string = `
{
    "raw_marker": 195,
    "basic_type": "Bool",
    "data": {
        "type": "Bool",
        "value": true
    }
}`;

let array: Uint8Array = pack_json(rmppJson);
console.assert(array[0] == 0xC3);
```

---

## Json format 🗃️
Previous unpack examples produce a json string that looks something like the following. It preserves all of the important metadata you might need. The `pack_json` method operates on json strings formatted like that.
```json
{
  "raw_marker": 130,
  "basic_type": "Map",
  "data": {
    "type": "FixMap",
    "value": [
      [
        {
          "raw_marker": 163,
          "basic_type": "String",
          "data": {
            "type": "FixStr",
            "value": "int"
          }
        },
        {
          "raw_marker": 1,
          "basic_type": "Number",
          "data": {
            "type": "FixPos",
            "value": 1
          }
        }
      ],
      [
        {
          "raw_marker": 165,
          "basic_type": "String",
          "data": {
            "type": "FixStr",
            "value": "float"
          }
        },
        {
          "raw_marker": 203,
          "basic_type": "Number",
          "data": {
            "type": "F64",
            "value": 1.0
          }
        }
      ]
    ]
  }
}
```

That's about it!