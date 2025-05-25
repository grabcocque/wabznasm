# Chapter 7: Jupyter Kernel Low-Level Implementation

This chapter delves into the low-level implementation details of the Wabznasm Jupyter kernel. It covers the communication mechanisms, message processing pipeline, and core components that enable interactive code execution within the Jupyter ecosystem.

## Core Components

The Wabznasm Jupyter kernel implementation is primarily structured around the following Rust modules and key structs within the `src/jupyter/` directory:

1.  **`kernel.rs` (`JupyterKernelRunner`)**:
    *   Manages the ZeroMQ (ZMQ) sockets for communication with the Jupyter front-end (Shell, IOPub, Heartbeat).
    *   Listens for incoming messages on the Shell socket.
    *   Parses raw ZMQ messages into structured `ParsedMessage` objects.
    *   Dispatches requests to the `WabznasmJupyterKernel` (handler).
    *   Constructs and sends replies back on the Shell socket.
    *   Handles the heartbeat (HB) socket communication.

2.  **`handler.rs` (`WabznasmJupyterKernel`)**:
    *   Implements the core logic for handling Jupyter messages (e.g., `kernel_info_request`, `execute_request`, `shutdown_request`).
    *   Interacts with the `JupyterSession` to evaluate code.
    *   Formats replies and results for the Jupyter front-end.
    *   Constructs and sends messages (like `status`, `execute_result`, `error`) on the IOPub socket for broadcast.

3.  **`session.rs` (`JupyterSession`)**:
    *   Maintains the state of the Wabznasm evaluator across multiple cell executions.
    *   Wraps an `Environment` from the core `wabznasm` evaluator.
    *   Parses and evaluates code snippets received from `execute_request` messages using the persistent environment.
    *   Tracks the execution count.

4.  **`message_parser.rs` (`ParsedMessage`)**:
    *   Responsible for deserializing raw multi-part ZeroMQ messages into a structured `ParsedMessage`.
    *   Handles message framing (identities, delimiter, signature, header, parent_header, metadata, content).
    *   Works in conjunction with `SignatureVerifier` to authenticate messages.

5.  **`signature.rs` (`SignatureSigner`, `SignatureVerifier`)**:
    *   Implements HMAC-SHA256 signing and verification of messages as per the Jupyter message specification.
    *   Ensures the integrity and authenticity of messages exchanged between the kernel and clients.

6.  **`connection.rs` (`ConnectionConfig`)**:
    *   Loads and stores the connection information (ports, IP, transport protocol, signature key) from the `kernel-*.json` file provided by Jupyter.

7.  **`errors.rs`**:
    *   Defines custom error types and type aliases (e.g., `JupyterResult`, `BoxedError`) used throughout the jupyter module for robust error handling.
    *   Includes `JupyterErrorFormatter` to convert internal `EvalError`s into the format expected by Jupyter front-ends for display.

8.  **`display.rs` (`JupyterDisplay` trait, `DisplayFormatter`)**:
    *   Formats Wabznasm `Value` types into various representations (e.g., `text/plain`, `text/html`) for rich display in Jupyter cells.

## Communication Flow (ZeroMQ)

The kernel communicates with the Jupyter client (e.g., Notebook, Lab, Console) over several ZeroMQ sockets:

*   **Shell Socket (Router/Dealer pattern)**:
    *   Receives requests from clients (e.g., `execute_request`, `kernel_info_request`).
    *   Sends replies back to the specific client that made the request.
    *   `JupyterKernelRunner` binds a `RouterSocket` and handles `recv()` and `send()` operations.

*   **IOPub Socket (Pub/Sub pattern)**:
    *   Used by the kernel to broadcast side effects to all connected clients. This includes:
        *   Output from code execution (`stdout`, `stderr` - though Wabznasm focuses on `execute_result`).
        *   `execute_result` containing the data to be displayed.
        *   Kernel status messages (`busy`, `idle`).
        *   Error messages from evaluation.
    *   `WabznasmJupyterKernel` (via `handler.rs`) sends messages on this socket.

*   **Heartbeat Socket (Rep/Req pattern)**:
    *   Used to monitor the kernel's liveness. The kernel simply echoes back any data received on this socket.
    *   Handled by `JupyterKernelRunner`.

*   **Control Socket (Router/Dealer pattern, similar to Shell)**:
    *   Can be used for messages like `shutdown_request` and `interrupt_request` if not handled via the Shell channel. (Currently, shutdown seems to be handled by Shell in `kernel.rs`).
*   **Stdin Socket**: Not typically used by kernels unless they require direct input from the user during code execution, which is rare for non-REPL style kernels. Wabznasm does not appear to use this.

## Message Structure and Processing

Jupyter messages follow a well-defined structure:

1.  **Message Identities (ZMQ Routing)**: One or more frames identifying the originating client (for ROUTER sockets).
2.  **Delimiter `<IDS|MSG>`**: Separates identities from the signed message parts.
3.  **HMAC Signature**: Hex-encoded signature of subsequent message parts.
4.  **Header (JSON string)**: Contains message ID, session ID, username, timestamp, message type, and protocol version.
5.  **Parent Header (JSON string)**: Header of the originating message, if this is a reply or a side effect.
6.  **Metadata (JSON string)**: Arbitrary metadata. Often an empty dictionary.
7.  **Content (JSON string)**: The actual payload of the message, specific to the `msg_type`.

**Processing Pipeline in `JupyterKernelRunner`:**

1.  **Receive Raw Message**: A `ZmqMessage` is received from the Shell socket.
2.  **Parse Message**: `ParsedMessage::parse()` is called:
    *   Frames are split.
    *   The `<IDS|MSG>` delimiter is located.
    *   Identities are extracted.
    *   The signature is extracted.
    *   Header, Parent Header, Metadata, and Content frames (as byte slices) are isolated.
    *   `SignatureVerifier::verify()` is called to check the HMAC signature against the relevant frames.
    *   If verification passes, the Header, Parent Header, Metadata, and Content frames are deserialized from JSON into their respective Rust structs (e.g., `jupyter_protocol::Header`, `jupyter_protocol::JupyterMessageContent`).
3.  **Dispatch to Handler**: Based on `header.msg_type`, the `ParsedMessage.content` (which is an enum like `JupyterMessageContent`) is matched:
    *   `KernelInfoRequest` -> `kernel_handler.kernel_info()`
    *   `ExecuteRequest` -> `kernel_handler.execute_request()` (this is `async`)
    *   `ShutdownRequest` -> `kernel_handler.shutdown_request()`
    *   Other types might be handled or logged as unhandled.
4.  **Handler Logic (`WabznasmJupyterKernel`)**:
    *   For `execute_request`:
        *   Sends `status: busy` on IOPub.
        *   Calls `self.session.execute(code)`.
        *   If `Ok(result)`:
            *   Formats the `result` using `JupyterDisplay` trait.
            *   If there's displayable data, constructs an `execute_result` IOPub message and sends it.
            *   Constructs an `ExecuteReply` with `status: ok`.
        *   If `Err(eval_error)`:
            *   Formats the `eval_error` using `JupyterErrorFormatter`.
            *   Constructs an `error` IOPub message and sends it.
            *   Constructs an `ExecuteReply` with `status: error`.
        *   Sends `status: idle` on IOPub.
        *   Returns the `ExecuteReply` to `JupyterKernelRunner`.
5.  **Construct Reply**: `JupyterKernelRunner` uses `construct_zmq_message()` (a global helper in `kernel.rs`) to assemble the reply:
    *   Serializes the reply Header, Parent Header (copied from request), Metadata, and Content into JSON byte strings.
    *   Signs these parts using `SignatureSigner::sign()`.
    *   Constructs a new `ZmqMessage` with identities, delimiter, new signature, and the serialized parts.
6.  **Send Reply**: The assembled `ZmqMessage` is sent back on the Shell socket.

**IOPub Message Construction (`construct_zmq_message_for_iopub` in `handler.rs`):**

This helper function is used by `WabznasmJupyterKernel` to create messages for the IOPub socket (like `status`, `execute_result`, `error`).
1.  Takes a `SimplifiedMessage` (a local struct in `handler.rs` holding a `Header`, `Option<Header>`, `metadata`, and `content`).
2.  Serializes these components to JSON byte strings.
3.  Signs them using the kernel's `SignatureSigner`.
4.  Constructs a `ZmqMessage`. Note: IOPub messages typically don't have identities prefixed as they are broadcast. The first frame is usually the topic (often derived from `message.header.msg_type`). The current implementation uses `message.header.msg_type.as_bytes().to_vec()` as the first frame.

## State Management (`JupyterSession`)

The `JupyterSession` is crucial for providing a stateful execution environment, mimicking how a user expects variables and function definitions to persist between Jupyter cells.

*   It holds an `Environment` from `wabznasm::environment`.
*   Each call to `session.execute(code)`:
    *   Increments an internal `execution_count`.
    *   Parses the `code` string using `tree_sitter` with the Wabznasm grammar.
    *   Checks for root-level parse errors (`root.has_error()`).
    *   If no parse error, it iterates through the top-level statements/expressions in the parsed code.
    *   For each statement, it calls `evaluator.eval_with_env(statement_node, code_string, &mut self.environment)`. The `&mut self.environment` is key, as it allows `eval_with_env` to modify the environment (e.g., by defining new variables or functions).
    *   The result of the *last* successfully evaluated statement is returned as `Option<Value>`. Assignments might conceptually return the assigned value or simply `None` if only side effects are considered.

## Error Handling

*   Parsing errors within `JupyterSession::execute` (e.g., `root.has_error()`) are converted into `EvalError` and propagated.
*   Evaluation errors from `evaluator.eval_with_env` are also `EvalError`.
*   In `WabznasmJupyterKernel::execute_request`, these `EvalError`s are caught.
*   `JupyterErrorFormatter::create_traceback` is used to generate a user-friendly traceback string array.
*   This information (`ename`, `evalue`, `traceback`) is sent as an `error` message on the IOPub socket.
*   The `ExecuteReply` message sent back on the Shell socket also indicates `status: error`.
*   The `JupyterResult` and `JupyterLocalResult` type aliases (using `Box<dyn std::error::Error + ...>`) are used for general operational errors within the Jupyter modules (e.g., socket errors, serialization errors).

## Key Takeaways for Low-Level Debugging/Development

*   **Message Signing is Critical**: Ensure the `key` from `connection.json` is correctly used by both `SignatureSigner` and `SignatureVerifier`. Mismatched signatures are a common source of messages being ignored or rejected.
*   **Message Framing**: The precise order and content of ZMQ message frames (identities, delimiter, signature, parts) must be strictly adhered to.
*   **Serialization/Deserialization**: Header, Parent Header, Metadata, and Content are all JSON. Ensure robust (de)serialization. Pay attention to how `Option<Header>` (for parent) and empty metadata/content are handled (often as empty JSON objects `{}`).
*   **IOPub Topic**: The first frame of an IOPub message is the "topic". The `jupyter_protocol` documentation or examples should clarify if using `msg_type` directly is standard or if it needs further prefixing (e.g., `execute_result.<msg_id>`). The current code uses `msg_type`.
*   **Asynchronous Operations**: Socket operations (`send`, `recv`) are asynchronous (`async/await`). The main `JupyterKernelRunner::run` loop is async, and `execute_request` in the handler is also async due to IOPub sends.

This chapter provides a foundational understanding of the kernel's internal workings, which should be helpful for anyone looking to debug issues, extend functionality, or understand its interaction with the Jupyter ecosystem at a deeper level.
