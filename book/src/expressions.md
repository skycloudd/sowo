# Expressions

Expressions are the base of the language. Every expression evaluates to a value.

The following are all valid expressions, with comments saying what value they evaluate to.

```swift
6 + 24
// -> 30

34 != 35
// -> true
//
// != means "not equals"

(1 + 2 * 3) > (1 * 2 + 3)
// -> true
//
// expressions can be as complex as you want!
```

## Data types

It's important to mention data types. Nova has these types.

-   int
-   float
-   bool
-   colour
-   vector

These are all the same types Exoracer has.

The type of an expression tells you what kind of value it will evaulate to. For example, the expression `1 + 2` has the type of `int` because it evaulates to a whole number (an integer). And the expression `(2 + 2) == 5` is of type `bool`, because it is a boolean (either `true` or `false`)

Strings are currently **not** implemented in the language. They will be added in the near future.

## Literals

Literals are the most basic expressions. They are the simple values like numbers and booleans.

### Integers

Integers are whole numbers. They can be positive or negative. They can be written in decimal, binary, or hexadecimal.

```swift
3
10
```

### Floats

Floats are decimal numbers. They can be positive or negative. They can be written in decimal, binary, or hexadecimal.

```swift
3.14
10.0
```

### Booleans

Booleans are either `true` or `false`.

```swift
true
false
```

### Colours

Colours are represented as hex codes. They are written as `#rrggbbaa` where `rr` is the red value, `gg` is the green value, and `bb` is the blue value. `aa` is an **optional** value for the alpha channel (opacity). Each value is a hexadecimal number between `00` and `ff` (0 and 255 in decimal).

```swift
#ff0000 // red
#00ff007f // green, with half opacity
#0000ff00 // blue, but completely transparent
#ff77a8 // pink
```

Alpha defaults to `ff` if it is not specified.

### Vectors

A vector is a pair of numbers. Vectors are written as `{ x, y }`. The values can only be of type `float`.

```swift
{ 1.0, 5.2 }
{ 0.123, 0.75 }
```

## Operators

Operators combine or modify values to create a new value. There are two kinds of operators: unary, and binary operators. Unary operators work on one value, while binary operators work on two values.

### Unary operators

```swift
<op><expr>
// -5
// -(22 + 3)
// !true
```

| Operator | Types      | Description                              |
| -------- | ---------- | ---------------------------------------- |
| `-`      | int, float | negates a number, multiplying it by `-1` |
| `!`      | bool       | inverts a boolean                        |

### Binary operators

```swift
<expr> <op> <expr>
// 5 + 6
// 11 - 3
// 2 > 1
```

| Operator | Types            | Description                                            |
| -------- | ---------------- | ------------------------------------------------------ |
| `*`      | int, float       | multiplies two numbers                                 |
| `/`      | int, float       | divides two numbers                                    |
| `+`      | int, float       | adds two numbers                                       |
| `-`      | int, float       | subtracts two numbers                                  |
| `>=`     | int, float       | checks if a number is greater than or equal to another |
| `<=`     | int, float       | checks if a number is less than or equal to another    |
| `>`      | int, float       | checks if a number is greater than another             |
| `<`      | int, float       | checks if a number is less than another                |
| `==`     | int, float, bool | checks if two values are equal                         |
| `!-`     | int, float, bool | checks if two values are not equal                     |

## Operator precedence

The operators above are listed in order of precedence. This means that operators higher in the list bind stronger than operators lower in the list. For example, `*` binds stronger than `+`, so `1 + 2 * 3` is the same as `1 + (2 * 3)`, instead of `(1 + 2) * 3`. Similarly, `+` binds stronger than `==`, so `1 + 2 == 3` is the same as `(1 + 2) == 3`.