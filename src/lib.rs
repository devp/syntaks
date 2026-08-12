/*
 * syntaks, a TEI Tak engine
 * Copyright (c) 2026 Ciekce
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

//! syntaks as a library.
//!
//! The modules are the same ones the `syntaks` binary is built from; the binary
//! is now a thin wrapper that calls [`tei::run`]. Exposing them lets the engine
//! be embedded directly by a host process, rather than only driven as a
//! subprocess over TEI — which some platforms, Android among them, do not permit.

pub mod bitboard;
pub mod board;
pub mod core;
pub mod correction;
pub mod eval;
pub mod history;
pub mod hits;
pub mod keys;
pub mod limit;
pub mod movegen;
pub mod movepick;
pub mod node_counter;
pub mod perft;
pub mod road;
pub mod search;
pub mod takmove;
pub mod tei;
pub mod thread;
pub mod ttable;
pub mod util;
