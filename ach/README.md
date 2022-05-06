# Ach

## Features

- `const`: static friendly
- `no_std`: Can run in embedded devices
- `no_alloc`: Needn't dynamic memory allocation
- Lock Free
- Wait Free: `try_send`/`try_recv` is Wait Free
- Spin: `send`/`recv` is only spin in critical section

## Usage

### AchOption

It can `set`/`take`/`replace`.

### Pool

It is an array of `AchOption`.

### Cell

It has allthe functions of `AchOption`, and it can be referenced.

It is similar to RwLock.

### Array

It is an array of `Cell`.

### Spsc

bounded SPSC queue.

### Ring

bounded ring buffer.

### Mpmc

bounded MPMC queue.

### Pubsub

broadcast channel.
