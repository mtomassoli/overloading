# Manual Trait Overloading

## What

Coming from C++, I was surprised to learn that Rust doesn't support *function overloading*. In this article I'll present a simple workaround.

As far as I can tell:

* Rust has powerful type inference (an extension of Hindley–Milner).
* Supporting function overloading in Rust would make type inference intractable:
  * Solution 1: drop overloading, like Rust did.
  * Solution 2: weaken type inference, like Scala did.
* As opposed to C++, Rust does type checking *before* monomorphization:
  * In Rust, monomorphization produces valid code for all valid concrete types.
    * Pros:
      * Better error messages.
      * Better separation between generic code and concrete types.
      * More predictable compilation times.
  * In C++, every single template instantiation is type-checked and can fail.
    * Pro:
      * More powerful.

I mentioned the difference between Rust's monomorphization and template instantiation because it informs the workaround discussed in this article.

Suppose that:

* A function `f` takes an argument `x` that can implement either `Trait1` or `Trait2`.
* `f` executes different code depending on whether `x` implements `Trait1` or `Trait2`.

I say that `f` is *trait-overloaded*.

If we add the requirement that the caller has to explicitly indicate the trait that `f` must use for `x`, then we have *manual* trait overloading.

Here's an example:

```rust
f(AsTrait1(&x1));
f(AsTrait2(&x2));
f(AsTrait2(&x1));
```

In the code above, we assume that `x1` implements both traits, so we can select either one for `x1`.

To simplify the language, I'll say that in the first call `x1` *uses* `Trait1`, and so on. That makes sense, since it's not really about what traits `x1` implements, but about what traits are effectively being used by `f`.

## How

The idea is simple:

* Let's suppose we're the developers of `f`.
* We define two wrapper types `AsTrait1` and `AsTrait2`, which wrap types that implement `Trait1` and `Trait2`, respectively.
* `f`'s signature is `fn f<T: AsTrait1Or2>(x: T) -> ...`.
* `AsTrait1` and `AsTrait2` implement the trait `AsTrait1Or2`, so they can be passed to `f`.
* `AsTrait1Or2` contains:
  * `TRAIT`: an ***associated const*** *enum* that indicates which trait to use
  * `t1`: a function that returns an `impl Trait1`
  * `t2`: a function that returns an `impl Trait2`
* In the `AsTrait1` implementation of `AsTrait1Or2`:
  * `TRAIT` indicates `Trait1`
  * `t1` returns the wrapped object as an `impl Trait1`
  * `t2` returns a dummy `impl Trait2`
* In the `AsTrait2` implementation of `AsTrait1Or2`:
  * `TRAIT` indicates `Trait2`
  * `t1` returns a dummy `impl Trait1`
  * `t2` returns the wrapped object as an `impl Trait2`
* `f` is implemented as follows, in general:

  ```rust
  // pseudo-code
  pub fn f<T: AsTrait1Or2>(x: T) -> ... {
      match T::TRAIT {
          Trait1 => f1(x),
          Trait2 => f2(x)
      }
  }
  ```

I was deliberately vague about the exact types because one can use immutable references, mutable references, or whatever is appropriate depending on the situation.

I haven't explored macros yet, but I imagine they could improve the ergonomics for the implementer of `f`.

## Why

The reason for using that scheme can be understood as follows:

* After monomorphization, we know (the concrete type for) `T`.
* Once we know `T`, we also know `T::TRAIT`, since it's an ***associated const***.
* Once we know `T::TRAIT`, we know which branch of the match to take.
* Once we know which branch to take, we can optimize the others away.
* Therefore, the function above will be monomorphized and optimized into `f1` or `f2` depending on the trait used.

Only *safe* code is used, so no undefined behavior is possible.

However, it's possible to use the wrong trait in a trait-overloaded function implementation, leading to a *panic*.

Ideally, `const TRAIT` would not only *indicate* the correct type, but also *narrow* the type from `Trait1 | Trait2` to either `Trait1` or `Trait2`:

```rust
// pseudo-code
pub fn f<T: AsTrait1Or2>(x: T) -> ... {
    match T::TRAIT {
        Trait1(x) => f1(x),     // x: Trait1
        Trait2(x) => f2(x)      // x: Trait2
    }
}
```

Unfortunately, that would make `TRAIT` *non-const*, so this is the best we can do:

```rust
pub fn f<T: AsTrait1Or2>(x: T) -> ... {
    match T::TRAIT {
        Trait1 => f1(x),
        Trait2 => f2(x)
    }
}
```

This opens us up to mistakes like the following:

```rust
pub fn f<T: AsTrait1Or2>(x: T) -> ... {
    match T::TRAIT {
        Trait1 => f2(x),        // instead of f1
        Trait2 => f1(x)         // instead of f2
    }
}
```

That code will type-check at compile time because there's no type narrowing, but it will panic at runtime since we'll hit an `unimplemented!()` by using the dummy objects instead of the real ones.

This is the price we pay for requiring type checking to occur *before* monomorphization, when we have no reasonable way to accurately express our intent within the type system.

### Check

If you want to verify that the monomorphized code is correctly optimized, I suggest that you:

* Add the following to the `Cargo.toml` file:

  ```toml
  [profile.release]
  debug = true
  ```

* Add `#[inline(never)]` to the function to analyze.
  * This way, the monomorphized code is isolated and easy to identify.
* Compile the code in `release` mode, of course.

I use VSCode with CodeLLDB and a debug configuration (in `launch.json`) like the following:

```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug RELEASE executable 'TraitOverloading'",
    "cargo": {
        "args": [
            "build",
            "--release",
            "--bin=TraitOverloading",
            "--package=TraitOverloading"
        ],
        "filter": {
            "name": "TraitOverloading",
            "kind": "bin"
        }
    },
    "args": [],
    "cwd": "${workspaceFolder}"
}
```

Alternatively, compile with `cargo rustc --release -- --emit=llvm-ir` and look at the `.ll` file in `src/target/release/deps`.

I recommend the first method, as you can set breakpoints and step through the code. Keep in mind, though, that breakpoints are less reliable in release mode since the source code doesn't map to the asm code as cleanly as in debug mode.

## When

This section touches upon some considerations that may help determine *when* this method is worth the effort.

### Expressivity

If `f` has only a single `AsTrait` argument, then:

```rust
f(AsTrait1(&x1));
f(AsTrait2(&x2));
```

is no better than:

```rust
f_Trait1(&x1);
f_Trait2(&x2);
```

However, for multiple `AsTrait` arguments we'd need an exponential number of separate implementations, in general.

Note that (classical) native overloading wouldn't solve the problem either. We'd still have an exponential number of implementations:

```rust
fn f(x: &impl Trait1, y: &impl Trait1) { ... }
fn f(x: &impl Trait1, y: &impl Trait2) { ... }
fn f(x: &impl Trait2, y: &impl Trait1) { ... }
fn f(x: &impl Trait2, y: &impl Trait2) { ... }
```

By using a single implementation and something like `AsTrait1Or2`, we get to have a single implementation that handles all the cases, exploiting any independence between the arguments. For instance, instead of:

```rust
// pseudo-code
fn f<T1, T2, T3, T4>(x1: T1, x2: T2, x3: T3, x4: T4)
where
    T1: AsTrait1Or2,
    T2: AsTrait1Or2,
    T3: AsTrait1Or2,
    T4: AsTrait1Or2,
{
    match (T1::TRAIT, T2::TRAIT, T3::TRAIT, T4::TRAIT) {
        (Trait1, Trait1, Trait1, Trait1) => { ... }
        (Trait1, Trait1, Trait1, Trait2) => { ... }
        (Trait1, Trait1, Trait2, Trait1) => { ... }
        (Trait1, Trait1, Trait2, Trait2) => { ... }
        (Trait1, Trait2, Trait1, Trait1) => { ... }
        (Trait1, Trait2, Trait1, Trait2) => { ... }
        (Trait1, Trait2, Trait2, Trait1) => { ... }
        (Trait1, Trait2, Trait2, Trait2) => { ... }
        (Trait2, Trait1, Trait1, Trait1) => { ... }
        (Trait2, Trait1, Trait1, Trait2) => { ... }
        (Trait2, Trait1, Trait2, Trait1) => { ... }
        (Trait2, Trait1, Trait2, Trait2) => { ... }
        (Trait2, Trait2, Trait1, Trait1) => { ... }
        (Trait2, Trait2, Trait1, Trait2) => { ... }
        (Trait2, Trait2, Trait2, Trait1) => { ... }
        (Trait2, Trait2, Trait2, Trait2) => { ... }
    }
}
```

we *might* just have:

```rust
// pseudo-code
fn f<T1, T2, T3, T4>(x1: T1, x2: T2, x3: T3, x4: T4)
where
    T1: AsTrait1Or2,
    T2: AsTrait1Or2,
    T3: AsTrait1Or2,
    T4: AsTrait1Or2,
{
    match T1::TRAIT {
        Trait1 => { ... }
        Trait2 => { ... }
    }
    match T2::TRAIT {
        Trait1 => { ... }
        Trait2 => { ... }
    }
    match T3::TRAIT {
        Trait1 => { ... }
        Trait2 => { ... }
    }
    match T4::TRAIT {
        Trait1 => { ... }
        Trait2 => { ... }
    }
    
    // shared code
    ...
}
```

...where the 4 matches do just some light work and most of the code is shared. In practice, we'll probably have something in the middle.

### Generality

#### Constraints

The constraints on the traits can even involve multiple arguments by passing tuples of arguments. For instance, a function could take two arguments and require that one argument use `Trait1` and the other `Trait2`:

```rust
f_xor((AsTrait1(&t1), AsTrait2(&t2)));
f_xor((AsTrait2(&t2), AsTrait1(&t1)));
```

The following calls would both fail, since the arguments use the same trait:

```rust
f_xor((AsTrait1(&t1), AsTrait1(&t2)));
f_xor((AsTrait2(&t1), AsTrait2(&t2)));
```

We could even require that *at least one* argument implement `Trait2`. There's no limit to the constraints we can express.

#### Foreign Traits

This method can certainly be used for traits defined in other crates. There are no problems with the *orphan rule*.

## Closing Words

Although I had a brief encounter with Rust years ago, I've only just started having a serious look at it, so I'm extremely welcoming of constructive thoughts and corrections.

Happy coding!
