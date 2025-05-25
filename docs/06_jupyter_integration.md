# Chapter 6: Jupyter Integration

## Overview

The Wabznasm language can be used interactively in Jupyter environments via a custom kernel. This chapter describes how to install, configure, and use the Wabznasm kernel with JupyterLab and Jupyter Notebook, as well as the features and limitations of the integration.

---

## 1. Installing and Registering the Kernel

### Prerequisites

- JupyterLab or Jupyter Notebook installed (via pip, conda, or Homebrew)
- Wabznasm built and available as an executable

### Registering the Kernel

After building Wabznasm, register the kernel with Jupyter:

```bash
wabznasm jupyter install
```

This will create a kernel spec in your Jupyter kernels directory, making "Wabznasm" available as a kernel option.

---

## 2. Using the Wabznasm Kernel

### Launching JupyterLab

```bash
jupyter lab
```

- In the launcher, select **Wabznasm** as the kernel for a new notebook.
- You can also change the kernel of an existing notebook via the Kernel menu.

### Launching Jupyter Console

```bash
jupyter console --kernel=wabznasm
```

---

## 3. Supported Features

- **Expression Evaluation:** All arithmetic, assignment, and function features described in the language reference.
- **Function Definitions and Calls:** Define and call functions interactively.
- **Persistent State:** Variables and functions persist across cells.
- **Error Reporting:** Errors are displayed inline in the notebook.
- **Kernel Info:** The kernel responds to Jupyter's info and status requests, enabling smooth integration.

---

## 4. Limitations

- **No Plotting/Graphics:** Only text output is currently supported.
- **No Rich Display:** Output is plain text; no HTML/Markdown rendering yet.
- **No Interactive Widgets:** Jupyter widgets are not supported.
- **No Multi-language Support:** Only Wabznasm code is supported in this kernel.

---

## 5. Troubleshooting

### Common Issues

- **Kernel Not Appearing:** Ensure you ran `wabznasm jupyter install` and that the kernel spec is in your Jupyter kernels directory.
- **Kernel Stuck on Startup:** Check the console output for errors. Ensure the wabznasm executable is built and accessible.
- **Cells Not Executing:** Make sure the kernel is selected and running. Check for error messages in the notebook and the terminal.

### Debugging Tips

- Run JupyterLab from a terminal to see kernel logs.
- Use the `wabznasm` REPL directly to test code outside Jupyter.
- If you see repeated `kernel_info_reply` messages, ensure the kernel sends IOPub status messages as described in the implementation guide.

---

## 6. Example Usage

### Basic Arithmetic

```wabz
2 + 3 * 4      // Returns 14
```

```wabz
2 ^ 3          // Returns 8 (exponentiation)
```

```wabz
5!             // Returns 120 (factorial)
```

### Variable Assignment

```wabz
x: 42
y: x + 8
y              // Returns 50
```

### Simple Function Definition

Define a function that adds one to its input:

```wabz
increment: {[x] x + 1}
```

Call the function:

```wabz
increment[5]   // Returns 6
```

```wabz
increment[42]  // Returns 43
```

### Multi-Parameter Functions

Define a function that takes two parameters:

```wabz
add: {[x;y] x + y}
```

```wabz
add[10; 20]    // Returns 30
```

```wabz
multiply: {[a;b] a * b}
multiply[6; 7] // Returns 42
```

### Function Composition

Functions can use other functions:

```wabz
double: {[x] x * 2}
quadruple: {[x] double[double[x]]}
```

```wabz
quadruple[5]   // Returns 20
```

### Using Variables Across Cells

Variables and functions persist across notebook cells:

**Cell 1:**
```wabz
base: 10
multiplier: 3
```

**Cell 2:**
```wabz
scale: {[x] x * multiplier}
```

**Cell 3:**
```wabz
result: scale[base]
result             // Returns 30
```

### Complex Function Examples

**Mathematical functions:**

```wabz
square: {[x] x * x}
cube: {[x] x * x * x}
```

```wabz
square[4]      // Returns 16
cube[3]        // Returns 27
```

**Compound calculations:**

```wabz
hypotenuse: {[a;b] square[a] + square[b]}
```

```wabz
hypotenuse[3; 4]  // Returns 25
```

### Function Closures

Functions capture their environment:

```wabz
offset: 100
makeAdder: {[n] {[x] x + n + offset}}
```

```wabz
add5: makeAdder[5]
add5[10]       // Returns 115 (10 + 5 + 100)
```

### Error Handling

**Syntax errors:**
```wabz
f: {[x] x +}   // Error: Syntax error in input
```

**Runtime errors:**
```wabz
f: {[x] x + y}
f[5]           // Error: Undefined variable: y
```

**Type/arity errors:**
```wabz
add: {[x;y] x + y}
add[5]         // Error: Arity mismatch: expected 2 arguments, got 1
```

### Debugging Functions

You can inspect intermediate values:

```wabz
debug_calc: {[x]
  step1: x * 2
  step2: step1 + 10
  step2
}
```

```wabz
debug_calc[5]  // Returns 20
```

### Working with State

Since the kernel maintains state, you can build up complex calculations:

**Cell 1:** Set up data
```wabz
prices: [100; 200; 150]  // Note: lists not yet implemented
count: 3
```

**Cell 2:** Define processing functions
```wabz
average: {[total; count] total / count}
discount: {[price] price * 0.9}
```

**Cell 3:** Use them together
```wabz
total: 450  // sum of prices
avg_price: average[total; count]
discounted: discount[avg_price]
discounted  // Returns 135.0
```

---

## 7. Developer Notes

- The kernel is implemented in Rust in `src/jupyter/`.
- The main entry point is `kernel.rs`.
- The kernel communicates with Jupyter using the ZeroMQ protocol and the Jupyter messaging spec v5.3.
- See `src/jupyter/handler.rs` for message handling logic.
- For protocol details, see the [Jupyter Messaging Protocol](https://jupyter-client.readthedocs.io/en/latest/messaging.html).

---
