## Punctuation-Only Core Operators

Applonti operators use only ASCII punctuation for maximum terseness:

- **Arithmetic / Unary:**

  - `+` (add)
  - `-` (subtract / negate)
  - `*` (multiply)
  - `/` (divide)
  - `%` (reciprocal)
  - `^` (power / exponent)

- **Reductions (Over `/`):**

  - `+/` (sum)
  - `*/` (product)
  - `&/` (minimum)
  - `|/` (maximum)
  - `&/` (all)
  - `|/` (any)

- **Scans (Prefix `\`):**

  - `+\` (prefix-sum)
  - `*\` (prefix-product)
  - `&\` (prefix-min)
  - `|\` (prefix-max)

- **Structural / Indexing / Application:**

  - `#` (take / count)
  - `_` (drop)
  - `$` (cast)
  - `?` (find / roll / enum-extend / fill)
  - `@` (apply / index / amend)
  - `.` (value / lookup)
  - `!` (key / dict / enum)
  - `,` (join / enlist)
  - `,/` (raze)
  - `^` (fill / coalesce)

- **Assignment / Amend:**

  - `:` (local assign / return)
  - `::` (global assign / alias)
  - `@:` (amend at)

- **Comparison:**

  - `=` (eq)
  - `<>` (ne)
  - `<` (lt)
  - `>` (gt)
  - `<=` (le)
  - `>=` (ge)
  - `~` (match)
  - `!~` (not-match)

- **Logical:**

  - `&` (and)
  - `|` (or)
  - `!` (not)

- **Pipeline / Composition:**

  - `|>` (left-to-right pipeline)

- **Iterators (Adverbs):**
  - `'` (each)
  - `/` (over)
  - `\` (scan)
  - `/:` (each right)
  - `\:` (each left)
  - `':` (each prior)

# wabznasm Reference Card (Markdown)

wabznasm is an array processing language inspired by Q/KDB+ and APL and J and Joy, that uses memory-mapped splayed trees based on Arrow and MsgPack for underlying storage

## Datatypes [4, 5, 8]

wabznasm uses numeric codes (short integers) to represent types. The `type` function returns a negative short for atoms and a positive short for lists/composites. [5]

- **Atoms (Negative Type ID):** [5]
  - `b`: boolean (`-1h`)
  - `g`: guid (`-2h`)
  - `x`: byte (`-4h`)
  - `h`: short (`-5h`)
  - `i`: int (`-6h`)
  - `j`: long (`-7h`, default integer type) [4]
  - `e`: real (`-8h`)
  - `f`: float (`-9h`, default float type) [3]
  - `c`: char (`-10h`)
  - `s`: symbol (`-11h`) [8]
  - `p`: timestamp (`-12h`)
  - `m`: month (`-13h`)
  - `d`: date (`-14h`)
  - `z`: datetime (`-15h`)
  - `n`: timespan (`-16h`)
  - `u`: minute (`-17h`)
  - `v`: second (`-18h`)
  - `t`: time (`-19h`)
- **Nulls & Infinities:** [4, 7]
  - Null: `0N` + type char (e.g., `0Ni`, `0Nj`, `0Nf`, `0And`), `" "`, `` ` `` (symbol null)Q
  - Infinity: `0W` + type char (e.g., `0Wi`, `0Wj`, `0Wf`, `0Wp`)
- **Lists & Composites (Positive Type ID):** [1, 4]
  - General List: `0h`
  - Typed List: `1h` to `19h` (e.g., long list is `7h`) [5]
  - Enum: `20h`-`76h` [4]
  - Dictionary: `99h` [4]
  - Table: `98h` [1, 4]
  - Functions/Operators/Iterators: `100h`-`111h` [4]

## Operators [11, 26]

Operators are built-in functions often used with infix notation. [21, 26]

- **Arithmetic:** `+`, `-`, `*`, `%` (Divide), `&` (Min), `|` (Max), `neg`, `reciprocal`, `swabznasmrt`, `exp`, `log`, `div`, `mod`
- **Comparison:** `=`, `<>`, `<`, `>`, `<=`, `>=`, `~` (Match), `like` [28]
- **Logical:** `&` (And), `|` (Or), `not`
- **Structural/Indexing/Application:**
  - `:` (Assign/Amend [26], Return [19])
  - `::` (Global Assign/Alias [26, 29])
  - `#` (Take/Count)
  - `_` (Drop/Cut/Floor) [25]
  - `$` (Cast [5]/Pad/String/Tok)
  - `?` (Find/Roll/Enum Extend/Fill) [25]
  - `@` (Apply/Index/Amend/Trap) [21, 23, 30]
  - `.` (Value/Amend At/Trap) [23, 30]
  - `!` (Key/Dict/Enum) [24]
  - `,` (Join/Enlist)
  - `^` (Fill/Join Nulls)
  - `(Right-to-left Pipeline) - Chains expressions:`f g x`means`f[g[x]]`
- **Iterators (Adverbs - see below):** `'`, `/`, `\`, `/:`, `\:` `'`

## Keywords (Selected Built-in Functions) [11, 14, 25]

These are named built-in functions.

- **Aggregation:** `avg`, `sum`, `prd`, `min`, `max`, `dev`, `var`, `med`, `cor`, `cov`, `scov`, `wavg`, `wsum`, `all`, `any` [14]
- **Searching/Sorting:** `asc` [14], `desc`, `iasc`, `idesc`, `rank`, `find` [25], `where`, `in`, `within`
- **Data Manipulation:** `select`, `update`, `delete`, `insert`, `upsert`, `exec`, `aj`, `asof`, `ij`, `lj`, `pj`, `uj`, `wj`, `cross`, `sublist`, `enlist`, `raze`, `flip`, `distinct`, `group`, `ungroup`, `xgroup`, `fby` [30]
- **Type/Metadata:** `type` [5], `key` [24], `value`, `meta`, `tables`, `views`, `cols` [14]
- **Math:** `abs`, `sqrt`, `exp`, `log`, `ceiling` [14], `floor`, `sin`, `cos`, `tan`, `acos` [25], `asin`, `atan`, `rand`
- **String/Text:** `string`, `lower`, `upper`, `trim`, `ltrim` [14], `rtrim`, `like`, `ss`, `ssr`, `sv`, `vs`, `parse`, `read0`, `read1`, `load` [14], `save`, `set`, `get` [19]
- **Temporal:** `date`, `time`, `month`, `year`, `hh`, `mm`, `ss`, `gtime`, `ltime`
- **Control/System:** `if`, `do`, `while`, `$[]` (Cond), `system` [20], `value`, `reval`, `peach`, `exit`, `getenv`, `setenv`
- **Documentation and Metadata:**
  - `.doc[fn; "docstring"]` - Adds documentation to a function
  - `.meta[fn; key; value]` - Associates metadata with a function

## Iterators (Adverbs) [11]

Applied as suffixes to verbs (operators/functions) to modify how they apply to lists.

- `'` (Each)
- `/` (Over)
- `\` (Scan)
- `/:` (Each Right)
- `\:` (Each Left)
- `':` (Each Prior)

## System Commands [2, 9, 11, 15]

Executed in the wabznasm console, prefixed with `\`.

- `\a`: List tables [6]
- `\b`: List views [6, 15]
- `\B`: List pending views [15]
- `\c`: Console size [2]
- `\C`: HTTP size [2]
- `\cd`: Change directory [2]
- `\d`: Show/set current namespace [9]
- `\e`: Error trap mode [2]
- `\f`: List functions [2]
- `\g`: Garbage collection mode [2]
- `\l`: Load script/directory [2, 10]
- `\o`: UTC offset [2]
- `\p`: Listening port [2, 9]
- `\P`: Print precision [9]
- `\r`: Replication primary / Rename file [9]
- `\s`: Number of secondary threads [2]
- `\S`: Random seed [2]
- `\t`: Timer interval [9]
- `\T`: Client query timeout [9]
- `\ts`: Time & space execution [9]
- `\u`: Reload user password file [2]
- `\v`: List variables [2]
- `\w`: Workspace info [2]
- `\W`: Week offset [2]
- `\x`: Expunge function/variable [2]
- `\z`: Date parsing format [9]
- `\1 <file>`: Redirect stdout [9]
- `\2 <file>`: Redirect stderr [9]
- `\_`: Hide wabznasm code [2]
- `\`: Terminate execution / Toggle wabznasm [2, 9]
- `\\`: Quit wabznasm session [2, 9]
- `system "cmd"`: Execute OS command via `system` keyword [15, 20]

## Attributes [11]

Applied using `` `# `` to lists or table columns.

- `` `g# ``: Grouped
- `` `p# ``: Parted
- `` `s# ``: Sorted
- `` `u# ``: Unique

## Namespaces [7, 11]

Used to organize variables and functions.

- `.`: Default global namespace
- `.q`: Standard wabznasm library functions
- `.Q`: Utility functions (I/O, IPC, etc.) [10, 22]
- `.h`: HTTP/HTML utilities
- `.j`: JSON serialization/deserialization
- `.z`: System/callback functions (e.g., `.z.ts`, `.z.ph`) [22]
- `.m`: Memory-backed file utilities [11]

## Constants [4, 7]

- Nulls: `0N?` (typed null, e.g., `0Ni`, `0Nj`), `` ` ``, `" "`, `(::)` (generic null)
- Infinities: `0W?` (typed infinity, e.g., `0Wi`, `0Wj`, `0Wf`)
- Booleans: `0b`, `1b`
- GUID: `0Ng`
- Empty Lists: `()`, `` `symbol$() `` [24], `0#0b`, `0#0j`, etc.

## Control Constructs [11]

- `if[condition; true_expr; ... ]`
- `do[count; expr; ... ]`
- `while[condition; expr; ... ]`
- `$[condition; true_expr; false_expr]` (Cond / ternary) [1]
- `:` (Signal/Return from function) [19]
- `'` (Signal error)

## Syntax Notes

- **Evaluation Order:** Right-to-left (or left-of-right). [21, 26]
- **Comments:** `/` for single line, `\` starts multi-line, `/` ends multi-line. [3]
- **Function Calls:** `f[arg1; arg2]` or `arg1 f arg2` (infix for dyadic) or `f arg` (prefix unary, brackets often optional). [26]
- **List Notation:** Items separated by spaces for numeric/temporal vectors [7]; enclosed in `()` for general lists; symbols prefixed with `` ` `` [7].
- **Assignment:** `:` for local/functional, `::` for global. [26, 29]
- **Namespaces:** Use `.` to separate parts (e.g., `.q.til`). [7]
- **Stack Effect Documentation**: Use the format `/ fn: (inputs -- outputs)` to document function inputs and outputs.
- **Local Definition Syntax:** Use `local` to define functions or variables in lexical scope.

## Function Documentation and Metadata

wabznasm supports enhanced function documentation that makes code more self-documenting and maintainable.

```q
/ Add docstring to a function
.doc[is_valid; "Validates time series data against constraints"]

/ Add metadata to a function that can be programmatically accessed
.meta[is_valid; `complexity; `O1]
.meta[is_valid; `performance; `fast]
.meta[is_valid; `author; `john]

/ Function with stack effect documentation in comment
/ filter_date: (table date -- filtered_table)
filter_date: {[t;d] select from t where date=d}

/ Local variable definition
process: {[data]
  local filter: {[x] x>0};
  local transform: {[x] x*2};
  transfor filter data
}
```

## Pattern Matching

wabznasm supports pattern matching for cleaner conditional logic:

```q
/ Pattern matching function
match: {[x]
  (x=0): {"Zero"};
  (x<0): {"Negative"};
  (x>100): {"Large"};
  _: {"Other"}
}

match[0]    / Returns "Zero"
match[-10]  / Returns "Negative"
match[150]  / Returns "Large"
match[50]   / Returns "Other"
```

## Combinatory Logic in wabznasm

wabznasm's function composition capabilities align with combinatory logic primitives. Using these combinators enables powerful point-free programming (tacit programming) without naming intermediate variables.

### Core Combinators

- **S' (S-prime)**: Function composition - `S'[f;g;h;x] ≡ f[g[x];h[x]]`

  ```q
  / Definition of S'
  S': {[f;g;h;x] f[g[x];h[x]]}

  / Example: Calculating average as (sum ÷ count)
  avg: sum S' (÷) count

  / Using S' to create a function that adds a value to its square
  addSquare: + S' (::) (*:)
  addSquare[3]  / Returns 12 (3 + 9)
  ```

- **K**: Constant function - `K[x;y] ≡ x`

  ```q
  / Definition of K
  K: {[x;y] x}

  / Example: Creating a function that always returns 42
  answer: K[42]
  answer[10]  / Returns 42
  ```

- **I**: Identity function - `I[x] ≡ x`

  ```q
  / Definition of I
  I: {x}

  / Using I as a passthrough
  2 + I[3]  / Returns 5
  ```

- **C**: Argument swap - `C[f;x;y] ≡ f[y;x]`

  ```q
  / Definition of C
  C: {[f;x;y] f[y;x]}

  / Example: Division with arguments reversed
  divideBy: C[%]
  divideBy[2;10]  / Returns 5 (10÷2)
  ```

- **W**: Argument duplication - `W[f;x] ≡ f[x;x]`

  ```q
  / Definition of W
  W: {[f;x] f[x;x]}

  / Example: Square a number using multiplication
  square: W[*]
  square[4]  / Returns 16 (4*4)
  ```

### Practical Applications

- **Filtering with S'**:

  ```q
  / Find all numbers greater than their square
  gtSquare: > S' (::) (*:)
  select from ([]n:1 2 3 4 5) where gtSquare each n  / Returns ([]n:1)
  ```

- **Composing Multiple Functions**:

  ```q
  / Using S' for complex transformations
  / Calculate (sum of squares) ÷ (count)
  avgSquare: (%) S' (sum S' (::) (*:)) count
  avgSquare[1 2 3 4 5]  / Returns 11 (55÷5)
  ```

- **Data Manipulation with Combinators**:

  ```q
  / Process a table with point-free notation
  / Sum each group divided by total sum
  relativeSum: (%) S' sum (sum@)
  select relSum:relativeSum[amount] by category from transactions
  ```

- **Tacit Programming Example**:

  ```q
  / Traditional function
  f1: {[x] (sum x) % count x}

  / Same function using combinators (point-free style)
  f2: (%) S' sum count

  / Both functions calculate the average
  f1[1 2 3 4 5]  / Returns 3
  f2[1 2 3 4 5]  / Returns 3
  ```

## Advanced Array Operations (APL/J-Inspired)

wabznasm extends the Q language with powerful array programming concepts from languages like APL and J, while maintaining ASCII syntax.

### Rank Polymorphism

Operations adapt automatically based on the rank (dimensionality) of arrays:

```q
/ Define a 3D array (2×3×2)
a: (2 3 2)$1+til 12

/ Sum across different ranks
+/a        / Sum across first dimension
+/"1 a     / Sum across second dimension
+/"2 a     / Sum across third dimension

/ Apply function with rank specification
f"n data   / Apply f with rank n
```

### Array Primitives

Extended array manipulation functions:

```q
/ Rotation (inspired by APL's rotate)
rotate: {[n;x] $[n>0; (n mod count x)_x,(n mod count x)#x; ((abs n) mod count x)#x,(-1*(abs n) mod count x)_x]}
1 rotate 1 2 3 4 5  / Returns 2 3 4 5 1

/ Reshape (inspired by APL's rho)
reshape: {[shape;data] shape $ (shape[0]*shape[1]*shape[2]) #data}
reshape[2 3;1 2 3 4 5 6]  / Creates a 2×3 matrix

/ Transpose (∙⍉ in APL, |: in J)
transpose: {flip x}
```

### J-Inspired Function Composition

Extended combinators for point-free programming:

```q
/ Hooks (dyad from monad and dyad): f@:g y ≡ y f (g y)
/ Definition: {[f;g;y] y f g y}
@:: {[f;g;y] y f g y}
mean: (+/)@:count  / Average using a hook
mean 1 2 3 4 5     / Returns 3

/ Forks (triad from three verbs): (f g h) y ≡ (f y) g (h y)
/ Definition: {[f;g;h;y] (f y) g (h y)}
FORK: {[f;g;h;y] (f y) g (h y)}
meansq: (+/ FORK * count) % count  / Mean of squares
meansq 1 2 3 4 5                   / Returns 11

/ Caps (dyad from two monads): f&:g y ≡ (f y) g y
/ Definition: {[f;g;y] (f y) g y}
&:: {[f;g;y] (f y) g y}
check: =&:type  / Check if value equals its type
```

### Advanced Tacit Programming

Combining functions without explicit arguments:

```q
/ Function trains (sequences of functions composed tacitly)
avg: (+/) % count        / Average as a train
rms: sqrt (+/ *: %) count / Root mean square
stddev: sqrt var         / Standard deviation

/ Point-free query expressions
/ Select records where value > average
filter: {[t;c] select from t where c > avg c}

/ Calculate ratio of each value to the sum
ratio: % S' (::) sum
```

### Workspace Model

```q
/ Save entire workspace
\save wsname.ws

/ Load workspace with all defined variables and functions
\load wsname.ws

/ List workspace contents
\vars
\fns
```

## Advanced Performance Features

wabznasm integrates high-performance concepts from several modern languages while maintaining Q-like terseness.

### Slicing and Vector Operations

Multi-dimensional array access with efficient memory layouts:

```q
/ Efficient array slicing with multiple dimensions
a: (5 5)$til 25  / 5×5 matrix

/ Slice syntax: array[dim1;dim2;...]
a[2;]      / Row 2 (returns 10 11 12 13 14)
a[;3]      / Column 3 (returns 3 8 13 18 23)
a[2 3;1 2] / Submatrix of rows 2-3, columns 1-2

/ Range slices
a[1+til 3;2+til 2]  / Rows 1-3, columns 2-3

/ Conditional slicing
a[where a[;1]>5;]   / Rows where column 1 > 5

/ Vectorized operations with rank specification
a +\: b      / Add each row of a to b
a +/: b      / Add each column of a to b
```

### Time-Series Optimizations

High-performance operations specialized for time-series data:

```q
/ Windowed operations (highly optimized internal implementation)
/ Rolling window calculations
mavg: {[w;x] w mavg x}          / Moving average
msum: {[w;x] w msum x}          / Moving sum
mmax: {[w;x] w mmax x}          / Moving maximum
mmin: {[w;x] w mmin x}          / Moving minimum
mdev: {[w;x] w mdev x}          / Moving standard deviation

/ Example: 3-period moving average
3 mavg 1 2 3 4 5 6              / Returns 0n 0n 2 3 4 5

/ Time-weighted and volume-weighted calculations
twap: {[times;prices] wavg[deltas times;prices]}
vwap: {[volumes;prices] wavg[volumes;prices]}
vwma: {[w;volumes;prices] w mvwma[volumes;prices]}  / Moving VWAP

/ Example: Volume-weighted average price
vwap[10 20 15 30;100 101 99 102]  / Returns 100.8

/ Efficient compression for time-series
.ts.compress[data;`delta]         / Delta encoding (store differences)
.ts.compress[data;`rle]           / Run-length encoding (for repeated values)
.ts.compress[data;`dict]          / Dictionary encoding for low-cardinality data

/ Asof join - the bread and butter of time-series analysis
aj[`time;trade;quote]             / Join quote data to trades on time
aj[`sym`time;trade;quote]         / Join quote data to trades on symbol and time
wj[windows;`time;trade;(quote;fills;{wavg[x;y]})]  / Window join with aggregation

/ Specialized date-time operations
update time: time.hh:mm:ss + 00:00:10 from trades  / Add 10 seconds
select from trades where time within 09:30:00 09:45:00  / Time range filter
update bucket: 5 xbar time.minute from trades     / 5-minute bucketing
```

### Multiple Dispatch and Type-Specific Optimizations

Runtime specialization based on argument types:

```q
/ Define multi-method with specialized implementations
/ Syntax: fn:{@[type x; type y] impl1; @[type x; type y] impl2;...;default}

/ Addition with type-specific optimizations
add:{
  @[`float; `float] {x+y};             / Float+Float: hardware FP add
  @[`int; `int] {`int$x+y};            / Int+Int: hardware int add
  @[`list; `atom] {x+\:y};             / List+Atom: vectorized add
  @[`atom; `list] {y+\:x};             / Atom+List: vectorized add
  @[`list; `list] {$[count[x]=count y; x+y; 'length]};  / List+List: zip add
  'type                                / Type error
}

/ Usage automatically dispatches to optimized implementation
add[1.0; 2.0]       / Uses floating-point path
add[1; 2]           / Uses integer path
add[1 2 3; 4]       / Uses list+atom path
```

### Context System

Implicit parameter passing with dynamic scoping:

```q
/ Define a context namespace
.ctx.db: (!) . flip (
    `conn`timeout`retries!
    (0Ni; 5000; 3)
);

/ Access context values with @. operator
query: {[sql]
    log[`debug; "Executing query with timeout: ", (string @.db.timeout)];
    exec_query[@.db.conn; sql; @.db.timeout]
}

/ Modify context for a code block using @@ operator
/ Syntax: fn @@ context
/ Or: {code block} @@ context_modifications
result: {
    query["SELECT * FROM special_table"];
    query["SELECT COUNT(*) FROM other_table"];
} @@ .ctx.db,`conn`timeout!(special_conn; 10000)

/ Function with bound context
with_logging: @@ [.ctx.log; `level`target!(`debug; `file)];
with_logging {
    log[`debug; "This message appears in the file"];
    perform_operation[];
}

/ Thread-local contexts
.Q.tp: {[ctx; f]     / thread with preserved context
    {[c;f] f @@ c}[ctx; f]
}

/ Thread function with preserved context
.Q.tp[.ctx.user,`id`role!(123; `admin)] peach items
```

### First-Class Data-Oriented Design

Memory layout control and data pattern optimizations:

```q
/ Data structure layout definition with packing hints
.struct.define[`Point; (
  (`x; `float; `align8);    / Aligned for SIMD
  (`y; `float; `align8)
)]

/ Create instances with optimal memory layout
p: .struct.new[`Point]
p.x: 1.0; p.y: 2.0

/ Column-wise storage for SoA (Structure of Arrays) approach
points: .struct.columnSet[`Point; 1000]  / Preallocate 1000 points
points.x: n?100.0                        / Fill x coordinates
points.y: n?100.0                        / Fill y coordinates

/ Optimized bulk operations on data structures
/ Process 1000 points in one vectorized operation
.struct.map[points; {[p] sqrt p.x*p.x + p.y*p.y}]
```

### Nil Handling in the Type System

First-class nil type with propagation rules:

```q
/ Nil value for each type
0N                 / Null integer
0n                 / Null float
`                  / Null symbol
::                 / Generic null

/ Nil-aware operators
x ?? y             / Return x if not null, otherwise y
x ?= y             / Check if x is null (without exception)
x !? f             / Apply f to x only if x is not null, else return null

/ Chaining with nil safety
/ Returns null if any path element is null without errors
user.settings.preferences.color

/ Nil-aware queries
/ Entries with null values in any field are excluded from result
select from table where not null? field

/ Optional type annotation (denoting nullable values)
fields: ((name; `symbol:?); (price; `float:?); (qty; `int))
```

### Seq Protocol and Transducers

Unified sequence operations inspired by Clojure but with Q-like syntax:

```q
/ Define a sequence - works on any sequential data structure
s: seq 1 2 3 4 5

/ Core sequence operations with point-free style
s |> each {x*2}                   / Map: 2 4 6 8 10
s |> filt {x>2}                   / Filter: 3 4 5
s |> redu {x+y} 0                 / Reduce: 15
s |> scan {x+y}                   / Running totals: 1 3 6 10 15
s |> take 3                       / First 3 elements: 1 2 3
s |> drop 2                       / Drop 2 elements: 3 4 5
s |> flat                         / Flatten nested sequences

/ Composition of operations using transducers
/ Define a transformation pipeline
xform: xdc[
    each {x*2};         / Double each value
    filt {x>5};         / Keep values > 5
    each {x-1}          / Subtract 1 from each
]the

/ Apply transformation to different collections with single pass
1 2 3 4 5 |> xform               / Returns 9 7 5
(1 2; 3 4; 5) |> xform           / Works on nested structures too

/ Efficient transformation with early termination
(1 2 3 4 5) |> take[3] xform  / Process just enough elements

/ Lazy evaluation with seq
infinite: seq {gen_next x} 1     / Infinite sequence starting at 1
infinite |> take 5               / Only computes the first 5 elements

/ Chunked operations for performance
data |> chunk 1000 xform      / Process in chunks of 1000

/ Stateful transducers
with_state: xdc[
    state["count" 0];            / Declare stateful transducer
    each {x + get_state["count"]}; / Use state in transformation
    upstate {inc_state["count"]} / Update state after each element
]
```

### Transients for Efficient Updates

Mutable collections for performance with controlled mutation scopes:

```q
/ Create a transient version of an immutable collection
t: tran (1 2 3 4 5)

/ Efficient updates using mutation
t: t +! 6                        / Append in-place
t: t *! 2                        / Multiply all elements in-place

/ Control mutation with scoping
v: with_tran (1 2 3) {           / Mutation allowed only in this block
    t +! 4;                      / Append 4
    t *! 2;                      / Double all elements
    t                            / Return the final transient
}                                / Automatically converts back to immutable
/ v is now (2 4 6 8) and immutable again

/ Persistent functional updates still work on immutable collections
v2: v + 10                       / Creates new immutable (2 4 6 8 10)
```

### Reactive Streams

Process asynchronous data streams with transducers:

```q
/ Define a reactive stream source
clicks: stream[`.z.ts]           / Create stream from timer events

/ Apply transformations using transducers
processed: clicks |> xdc[
    filt {[event] event.type=`click};  / Only process clicks
    each {[event] event.target};       / Extract target
    window 5;                         / Group into windows of 5
    each {[window] freq window}       / Count frequencies
]

/ Subscribe to results
on_stream[processed; {[freqs] update_ui freqs}]

/ Merge multiple streams
combined: merge_stream[mouse_stream; keyboard_stream]

/ Throttle high-frequency events
throttled: clicks |> throttle 100   / Max one event per 100ms

/ Back-pressure handling
with_backpressure: stream |> buffer 1000   xdc[
    filt valid?;
    each process;
]
```

## Memory Management and Performance Control

```q
/ Explicit memory hints
.mem.pin[x]         / Pin object in memory (prevent GC)
.mem.unpin[x]       / Allow object to be garbage collected
.mem.copy[x]        / Ensure a clean copy (for mutation)
.mem.profile[]      / Show memory usage by type
.mem.compact[]      / Force compaction of memory

/ Explicit vectorization hints
.opt.simd[expr]     / Request SIMD optimization
.opt.thread[expr;8] / Request threading with 8 threads
.opt.parallel[expr] / Auto-parallelize appropriate expressions
```

## Wabznasm

This section is reserved for future documentation about 'wabznasm'.
