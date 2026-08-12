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

use crate::bitboard::Bitboard;

/// Expands `set` to the squares orthogonally adjacent to it, restricted to `occ`.
///
/// Shifts of 6 move between ranks. Shifts of 1 move between files and are masked
/// against the edge they would wrap into, so a square on the right file cannot
/// leak onto the left file of the next rank.
#[must_use]
fn expand(set: u64, occ: u64) -> u64 {
    let up = set << 6;
    let down = set >> 6;
    let left = (set << 1) & !Bitboard::LEFT_EDGE.raw();
    let right = (set >> 1) & !Bitboard::RIGHT_EDGE.raw();

    (up | down | left | right) & occ
}

/// Flood fills outwards from `seed` through `occ` until it stops growing.
#[must_use]
fn flood(seed: u64, occ: u64) -> u64 {
    let mut set = seed;

    loop {
        let next = set | expand(set, occ);

        if next == set {
            return set;
        }

        set = next;
    }
}

/// Portable equivalent of the AVX2 and SSE4.2 implementations, for targets with
/// neither — notably aarch64.
///
/// Only one seed per axis is used. A road exists exactly when the fill from one
/// edge reaches the opposite edge, so filling from both ends as the SIMD versions
/// do is an iteration-count optimisation rather than a difference in meaning.
#[must_use]
pub(super) fn has_road(road_occ: u64, up: u64, _down: u64, left: u64, _right: u64) -> bool {
    flood(up, road_occ) & Bitboard::LOWER_EDGE.raw() != 0
        || flood(left, road_occ) & Bitboard::RIGHT_EDGE.raw() != 0
}
