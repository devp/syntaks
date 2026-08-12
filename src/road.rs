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

#[cfg(target_feature = "avx2")]
mod avx2;

// Compiled on every target rather than only those without SIMD, so that the
// SIMD implementations can be differentially tested against it.
#[cfg_attr(
    any(target_feature = "avx2", target_feature = "sse4.2"),
    allow(dead_code)
)]
mod scalar;

#[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.2"))]
mod sse;

use crate::bitboard::Bitboard;

#[must_use]
pub fn has_road(road_occ: Bitboard) -> bool {
    let upper_edge = Bitboard::UPPER_EDGE.raw();
    let lower_edge = Bitboard::LOWER_EDGE.raw();
    let left_edge = Bitboard::LEFT_EDGE.raw();
    let right_edge = Bitboard::RIGHT_EDGE.raw();

    let road_occ = road_occ.raw();

    let up = road_occ & upper_edge;
    let down = road_occ & lower_edge;
    let left = road_occ & left_edge;
    let right = road_occ & right_edge;

    let up = up | (up >> 6 & road_occ);
    let down = down | (down << 6 & road_occ);
    let left = left | (left << 1 & road_occ);
    let right = right | (right >> 1 & road_occ);

    #[cfg(target_feature = "avx2")]
    {
        //SAFETY: self-explanatory
        return unsafe { avx2::has_road(road_occ, up, down, left, right) };
    }

    #[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.2"))]
    {
        return unsafe { sse::has_road(road_occ, up, down, left, right) };
    }

    #[cfg(not(any(target_feature = "avx2", target_feature = "sse4.2")))]
    {
        return scalar::has_road(road_occ, up, down, left, right);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOARD: u64 = (1 << 36) - 1;

    /// Reproduces the seeding that `has_road` does before dispatching.
    fn seeds(road_occ: u64) -> (u64, u64, u64, u64) {
        let up = road_occ & Bitboard::UPPER_EDGE.raw();
        let down = road_occ & Bitboard::LOWER_EDGE.raw();
        let left = road_occ & Bitboard::LEFT_EDGE.raw();
        let right = road_occ & Bitboard::RIGHT_EDGE.raw();

        (
            up | (up >> 6 & road_occ),
            down | (down << 6 & road_occ),
            left | (left << 1 & road_occ),
            right | (right >> 1 & road_occ),
        )
    }

    /// Deliberately naive square-by-square search, written independently of the
    /// bitboard implementations so that agreement between them means something.
    fn reference_has_road(road_occ: u64) -> bool {
        fn reaches(road_occ: u64, from: impl Fn(usize) -> bool, to: impl Fn(usize) -> bool) -> bool {
            let mut seen = [false; 36];
            let mut stack: Vec<usize> = (0..36)
                .filter(|&sq| road_occ >> sq & 1 == 1 && from(sq))
                .collect();

            for &sq in &stack {
                seen[sq] = true;
            }

            while let Some(sq) = stack.pop() {
                if to(sq) {
                    return true;
                }

                let (rank, file) = (sq / 6, sq % 6);
                let mut neighbours = Vec::new();

                if rank > 0 { neighbours.push(sq - 6); }
                if rank < 5 { neighbours.push(sq + 6); }
                if file > 0 { neighbours.push(sq - 1); }
                if file < 5 { neighbours.push(sq + 1); }

                for n in neighbours {
                    if !seen[n] && road_occ >> n & 1 == 1 {
                        seen[n] = true;
                        stack.push(n);
                    }
                }
            }

            false
        }

        reaches(road_occ, |sq| sq / 6 == 5, |sq| sq / 6 == 0)
            || reaches(road_occ, |sq| sq % 6 == 0, |sq| sq % 6 == 5)
    }

    fn scalar_road(road_occ: u64) -> bool {
        let (up, down, left, right) = seeds(road_occ);
        scalar::has_road(road_occ, up, down, left, right)
    }

    /// xorshift64*, so the cases are reproducible without pulling in a dependency.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
        }

        /// ANDing randoms together sweeps the density, which is what actually
        /// varies whether a road is present and how near-miss the position is.
        fn board(&mut self, ands: u32) -> u64 {
            let mut board = BOARD;
            for _ in 0..ands {
                board &= self.next();
            }
            board & BOARD
        }
    }

    #[test]
    fn known_positions() {
        assert!(!scalar_road(0), "empty board has no road");
        assert!(scalar_road(BOARD), "full board has a road");
        assert!(scalar_road(Bitboard::LEFT_EDGE.raw()), "a full file is a road");
        assert!(scalar_road(Bitboard::LOWER_EDGE.raw()), "a full rank is a road");
        // A file with its middle square missing is not a road.
        assert!(!scalar_road(Bitboard::LEFT_EDGE.raw() & !(1 << 18)));
        // Wrapping from the right file to the left file of the next rank must
        // not count as adjacency.
        assert!(!scalar_road(Bitboard::RIGHT_EDGE.raw() & !(1 << 35) | 1 << 30));
    }

    #[test]
    fn scalar_matches_reference() {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);

        for ands in 1..=4 {
            for _ in 0..40_000 {
                let board = rng.board(ands);
                assert_eq!(
                    scalar_road(board),
                    reference_has_road(board),
                    "disagreement on board {board:#x}"
                );
            }
        }
    }

    #[cfg(target_feature = "avx2")]
    #[test]
    fn scalar_matches_avx2() {
        let mut rng = Rng(0xdead_beef_cafe_f00d);

        for ands in 1..=4 {
            for _ in 0..40_000 {
                let board = rng.board(ands);
                let (up, down, left, right) = seeds(board);
                let simd = unsafe { avx2::has_road(board, up, down, left, right) };
                assert_eq!(scalar_road(board), simd, "disagreement on board {board:#x}");
            }
        }
    }

    #[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.2"))]
    #[test]
    fn scalar_matches_sse() {
        let mut rng = Rng(0xdead_beef_cafe_f00d);

        for ands in 1..=4 {
            for _ in 0..40_000 {
                let board = rng.board(ands);
                let (up, down, left, right) = seeds(board);
                let simd = unsafe { sse::has_road(board, up, down, left, right) };
                assert_eq!(scalar_road(board), simd, "disagreement on board {board:#x}");
            }
        }
    }
}
