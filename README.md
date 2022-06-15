A fast, iterative, correct approach to Stackblur, resulting in a very smooth and
high-quality output, with no edge bleeding.

This crate implements a tweaked version of the Stackblur algorithm requiring
`radius * 2 + 2` elements of space rather than `radius * 2 + 1`, which is a
small tradeoff for much-increased visual quality.

The algorithm is exposed as an iterator (`StackBlur`) that can wrap any other
iterator that yields elements of `StackBlurrable`. The `StackBlur` will then
yield elements blurred by the specified radius.

## Benefits of this crate

Stackblur is essentially constant-time. Regardless of the radius, it always
performs only 1 scan over the input iterator and outputs exactly the same amount
of elements.

Additionally, it produces results that are comparable to slow and expensive
Gaussian blurs. As opposed to box blur which uses a basic rolling average,
Stackblur uses a weighted average where each output pixel is affected more
strongly by the inputs that were closest to it.

Despite that, Stackblur does not perform much worse compared to naive box blurs,
and is quite cheap compared to full Gaussian blurs, at least for the CPU. The
implementation in this crate will most likely beat most unoptimized blurs you
can find on crates.io, as well as some optimized ones, and it is extremely
flexible and generic.

For a full explanation of the improvements made to the Stackblur algorithm, see
the `iter` module.

https://user-images.githubusercontent.com/4723091/173788732-2e3e125e-f7b3-4e0f-8582-cc2c148ba437.mp4

*(In the above video, `stackblur-iter` is the centered blur, whereas the
full-width one is another `stackblur` crate.)*

## Comparison to the `stackblur` crate

`stackblur` suffers from edge bleeding and flexibility problems. For
example, it can only operate on buffers of 32-bit integers, and expects them
to be packed linear ARGB pixels. Additionally, it cannot operate on a 2D
subslice of a buffer (like `imgref` allows for this crate), and it does not
offer any streaming iterators or documentation. And it also only supports
a blur radius of up to 255.

## Usage

Aside from `StackBlurrable` and `StackBlur` which host their own documentation,
there are helper functions like `blur` and `blur_argb` that can be used to
interact with 2D image buffers, due to the fact that doing so manually involves
unsafe code (if you want no-copy).

See the [full documentation](https://docs.rs/stackblur-iter) for more.
