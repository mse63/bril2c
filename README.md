# Bril2C
This project is intended to translate code from Bril (Big Red Intermediate Language) to C. Bril is an educational intermediate language used in the CS6120 course at Cornell University. As an intermediate language, it is low-level, which would make it difficult to translate to a high-level language. However, C, with features such as manual memory management and goto statements, makes it a natural choice. In my initial proposal, I thought creating a complete and correct translation from Bril to C would be trivial, but indeed quite a bit of hacking was needed, due to limitations of C not placed on Bril.

## Implementation
Bril2C is written in Rust, using the "bril-rs" package. Most of the actual translation work happens in a single trait, `Crep`, which contains a single function `crep(self) -> String`. Each piece of Bril implements this function with how it translates itself to C, calling `crep` on smaller pieces, effectively inducting on the structure of the program. For example, `crep` for a program will, along with other things, call `crep` for each function in that program, which will call `crep` for each instruction, which will call `crep` for each operation and variable. In this way, adding additional features of Bril into C becomes as easy as implementing a single function for a specific type.

As it turns out, quite some fiddling is required to get even a majority of Bril code working. Almost all of the benchmarks have `main` function which takes in arguments, something not possible in `C`. We get around this by creating our own `main` function in C, which parses arguments from the command line, and passes them into a copy that's supposed to represent the Bril's main function, called `main_f`. Taking an example, `sum-bits.bril`, with a main function that takes in a single integer argument, this is represented as:
```C
int main(int argc, char *argv[]) {
  int64_t input = atoi(argv[1]);
  main_f(input);
  return 0;
}
```
This way the resulting program can take in arguments from the command line for the main function in the same way as `brili`.

Additionally, Bril's print statement isn't type-annotated, meaning some form of polymorphism is needed in C. We accomplish this by creating our own generic print macro, allowing us to call `print` as easily as it is called in Bril:
```C
#define print(x)                                                               
  _Generic((x),
      int64_t: printf("%" PRId64 " ", x),
      uint8_t: printf("%s ", (x) ? "true" : "false"),
      double: printf("%.17f ", x))
```
For ease of programming, each function declares all of the variables used in it at the beginning of the function After that, it's a "simple" translation from Bril instructions to C statements. Therefore, the following function in Bril:
```
@mod(dividend : int, divisor : int) : int {
  quotient : int = div dividend divisor;
  two : int = const 2;
  prod : int = mul two quotient;
  diff : int = sub dividend prod;
  ret diff;
}
```
becomes this C function:
```C
int64_t mod_f(int64_t dividend_, int64_t divisor_) {
  int64_t quotient_;
  int64_t two_;
  int64_t prod_;
  int64_t diff_;
  quotient_ = dividend_ / divisor_;
  two_ = 2;
  prod_ = two_ * quotient_;
  diff_ = dividend_ - prod_;
  return diff_;
}
```
At this point, you may have noticed the additional characters in each variable name and suffix name. These are inserted to prevent bril registers from being confused with C keywords, which is surprisingly popular in Bril code, especially for words like `if` and `continue`.

In order to capture the maximum subset of Bril, we also implemented the `memory` and `float` extensions to Bril. Adding these were surprisingly simple, as Bril's `alloc`, `free`, `load`, and `store` map rather painlessly to C's `malloc`, `free`, and referencing/dereferencing. To show a full example, `Bril2C` successfully translated this Bril code:
```
# ARGS: 42
@main(input : int) {
  sum : int = const 0;
  two : int = const 2;
  zero : int = const 0;
.loop:
  cond : bool = eq input zero;
  br cond .done .body;
.body:
  bit : int = call @mod input two;
  input : int = div input two;
  sum : int = add sum bit;
  jmp .loop;
.done:
  print sum;
  ret;
}

@mod(dividend : int, divisor : int) : int {
  quotient : int = div dividend divisor;
  two : int = const 2;
  prod : int = mul two quotient;
  diff : int = sub dividend prod;
  ret diff;
}
```
into this C code:
```C
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

uint8_t true = 1;
uint8_t false = 0;
#define print(x)                                                               \
  _Generic((x),                                                                \
      int64_t: printf("%" PRId64 " ", x),                                      \
      uint8_t: printf("%s ", (x) ? "true" : "false"),                          \
      double: printf("%.17f ", x))

void main_f(int64_t input_);

int64_t mod_f(int64_t dividend_, int64_t divisor_);

void main_f(int64_t input_) {
  int64_t sum_;
  int64_t two_;
  int64_t zero_;
  uint8_t cond_;
  int64_t bit_;
  sum_ = 0;
  two_ = 2;
  zero_ = 0;
loop_:
  cond_ = input_ == zero_;
  if (cond_) {
    goto done_;
  } else {
    goto body_;
  }
body_:
  bit_ = mod_f(input_, two_);
  input_ = input_ / two_;
  sum_ = sum_ + bit_;
  goto loop_;
done_:
  print(sum_);
  printf("\n");
  return;
}
int64_t mod_f(int64_t dividend_, int64_t divisor_) {
  int64_t quotient_;
  int64_t two_;
  int64_t prod_;
  int64_t diff_;
  quotient_ = dividend_ / divisor_;
  two_ = 2;
  prod_ = two_ * quotient_;
  diff_ = dividend_ - prod_;
  return diff_;
}
int main(int argc, char *argv[]) {
  int64_t input = atoi(argv[1]);
  main_f(input);
  return 0;
}
```

## Evaluation
Creating a program to translate a single Bril program into C is easy. Creating a program to translate _every_ bril program into C is much more difficult. I tested my code with a simple script, `test.sh` which would run every `.bril` file in `benchmarks`, and compare its output with the translated, compiled, and run code from `Bril2C`. Many times, I thought I had completed my project, only to discover a Bril program that broke my seemingly correct implementation, such as by naming a register the same name as a function, or by using the same register with two different types in two different functions. After enough bug-fixing, however, I was able to successfully produce a correct program for every single Bril program in the benchmark (as shown in `test_results.txt`).

## Future Work
I believe this project still has room for improvement. Although my code did pass every Bril program in the benchmark, it would not surprise me if someone could create a nefarious Bril program that Bril2C would fail with, perhaps by 

Additionally, I believe there is room for improvement in the generated C code. I'm not a fan of the use of `goto` in C, and I'm confident many if not most of the control logic in Bril code could be represented with `if` statements or `while` loops. Unfortunately, I did not end up having the time to explore this idea further.
