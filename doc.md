# BEL

BEL is garbage-collected language.

## Types
| Name           | Width in bits | Description                                      |
|:--------------:|:-------------:|:------------------------------------------------:|
| bool           | 8             | Logical value                                    |
| i8/i16/i32/i64 | 8/16/32/64    | Integer value with 8/16/32/64 bit width.         |
| u8/u16/u32/u64 | 8/16/32/64    | Unsigned integer with 8/16/32/64 bit width.      |
| char           | 32            | UTF scalar value.                                |

## Statements

### Declaration statements

`let VARIABLE: TYPE[ = VALUE];`

### Assignment statement

`VARIABLE = VALUE;`

### If statement

```
if CONDITION_EXPRESSION {
    BODY1
} [else {
    BODY2
}]
```

### While loop

### For loop
