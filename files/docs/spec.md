# Types

> **IMPORTANT:**  
> **Shard** provides no guarantees about the contents of types. The type system is purely based on
> memory size. As such, no clearing is done for any types, or casts.

## Sizes

A size in **Shard** is a non-cleared, contiguous span of bytes. Alignment of sizes is *not* guaranteed
by **Shard**, so using a power of 2 size may help improve cache locality. These can be used to
represent intagers, floats, booleans and any contiguous data. This is the only atomic type in **Shard**
and every other type provides useful metadata around sizes.   
Some examples include: `1`, `8`, `98`.

## Arrays

Arrays in **Shard** can provide additional metadata around a size, allowing indexing into it. 
An array is defined by the type of the element, followed by a `:`, and optionally a known size.
Arrays can also be unbounded when a maximum length is not provided, allowing access to any
element, even if it is outside of the array. *Unbounded arrays* must be **heap** allocated,
as the stack by nature does not allow dynamic sizes.

> **NOTE**:  
> When using an unbounded array, it is idiomatic to store its length, or store a sentinel value
> (as is the case for c-style strings). Indexing into unbounded arrays is not checked by
> **Shard** at compile time unlike sized arrays, so measures must be in place to ensure out of bounds
> memory access does not occur.

Array lengths can also be infered by using `?` in place for their length. This will let the **Shard**
compiler resolve the length of the array, provided that the value is of a known length. eg:

```
%three_colours {1:?} <- {`r`, `g`, `b`}
```

## Heaps

Heaps are a composite type allowing for the contiguous grouping of related data, without any
implicit padding between types. To define a heap, wrap any number of comma separated types
(including nested heaps) in either `{}` or `[]` (eg. `[1, 2]`, `{2:3, [2], 3}`).  
Trailing commas are also permitted eg. `{2, 2,}`.

Heaps wrapped in `[]` are a *pointer to a heap* and are functionally equivalent to `[{}]`,
with the addition that they can contain *unbounded arrays* (which provide no safety about indexing
out of bounds, unlike sized arrays).

Unlike other languages, order of heap members is guaranteed, meaning that alignment may be
inefficient depending on how the heap laid out. To combat this, **Shard's** pad token `..` may
replace a separating comma `,` to aid in padding heaps efficiently, without affecting indexing. The
pad character inserts un-indexable bytes until the largest item's size or the architectures word
size is reached. Take the example: `{1, 1, 4}` which is inefficient as third item is not 32bit
aligned. `{1, 1 .. 4}` (functionally equivalent to `{1, 1, 2, 4}`, however indexing skips the
inserted padding short).

> **CAUTION**:  
> When using a pointer to a heap (`[]`), note that **padding space needs to be accounted and
> allocated for**. This can be tricky to get right so it is recommend that the allocation
> is done using **Shard's** `@size` builtin, which will return the size of a type, with padding included

## Structs

Structs are a type alias for heaps, with added names for field access. They can be defined using
the `struct` keyword, an identifier, and a heap as normal, with a type preceding each identifier:
eg.

```
struct Position {
    4 x,
    4 y,
}
```

Members of structs are accessed with the member access operator `.` eg. `pos.x`. 
Padding also works here too:

```
struct MyPaddedStruct {
    1 a ..
    2 b ..
}
```

## Type Casting

// TODO: redo the type casting section
<!-- All operators in **Shard** have a type assigned to them. A type is infered when _only_ one side of -->
<!-- the operator is unknown: eg. `%str = "Hello, World!"` (type of `=` is infered from the string -->
<!-- literal, which then propagates to the type of `str`, resulting in `str` being `{1:13}`[^1]). This -->
<!-- only works for binary operators, unary operators cannot be type cast. -->
<!---->
<!-- Typically values already have types assigned to them, however a common example is type narrowing is -->
<!-- casting from a void pointer to a [Struct](#structs): -->
<!---->
<!-- ``` -->
<!-- %my_pos Position = $malloc @size Position -->
<!-- ``` -->
<!---->
<!-- This way of casting on operators allows easy casting of the result of an operation to a new -->
<!-- type, and also allows for explicit behaviour for overloaded operators eg: -->
<!---->
<!-- ``` -->
<!-- %start {4:2} = {2, 0} -->
<!-- %move {4:2} = {1, 1} -->
<!-- %end = start Position+ move -->
<!-- ``` -->
<!---->
<!-- In this example, start and move are both cast to Position before getting added with the position's -->
<!-- addition operator. Finally, the type of end is infered to be of Position, as there is exactly one -->
<!-- side known, and another unknown. Alternatively, you may also cast on the `=`, and you can chose -->
<!-- either (or both) as it just a matter of style. (eg. `%end Position = start + move`) -->
<!---->
<!-- > **IMPORTANT:**   -->
<!-- > One thing to note is that type casting is usually done between types of the same size, however -->
<!-- > in the case that the intention is to increase the memory space space of a type by casting to a -->
<!-- > larger type, it should be noted that **Shard** does not 0 initialize memory so care must be taken -->
<!-- > when resising intagers for instance, to make sure they are valid by 0-ing out memory. This -->
<!-- > problem does not arise when binding to a register, as the whole register is completely wiped when -->
<!-- > set to a new value. -->
<!---->
<!-- > **NOTE:**   -->
<!-- > When shrinking types, only the new, smaller size will get copied. This may be problematic, -->
<!-- > especially for strings, if the termination (if there is one) is not appropriately re-set after -->
<!-- > casting. -->
<!---->
<!-- [^1]: -->
<!--     This type is only `{1:13}` if the `:STRTERM` tag is not set, which is by default set to `0` -->
<!--     when using **Shard's** standard library. If `:STRTERM` is set, the infered size of string literals be -->
<!--     one longer to accomodate for the terminal character (eg. `{1:14}`) -->
