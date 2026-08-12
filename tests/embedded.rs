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

//! Drives the engine the way an embedding host does: through the library, with
//! no stdin to read commands from and no stdout to parse a result out of.

use std::time::Instant;

use syntaks::board::Position;
use syntaks::limit::Limits;
use syntaks::movegen;
use syntaks::search::{MAX_DEPTH, Searcher};
use syntaks::tei::TeiOptions;

fn search_for_nodes(pos: &Position, nodes: usize) -> Option<syntaks::takmove::Move> {
    let mut searcher = Searcher::new();
    let start = Instant::now();

    let mut limits = Limits::new(start);
    assert!(limits.set_nodes(nodes), "node limit rejected");

    searcher.start_search(
        pos,
        &[],
        start,
        limits,
        MAX_DEPTH,
        &[],
        &TeiOptions::default(),
    );
    searcher.wait();

    searcher.best_move()
}

#[test]
fn search_reports_a_best_move_through_the_library() {
    let pos = Position::startpos();
    let mv = search_for_nodes(&pos, 20_000).expect("no best move was recorded");

    let mut legal = Vec::new();
    movegen::generate_moves(&mut legal, &pos);

    assert!(
        legal.contains(&mv),
        "search returned {mv}, which is not legal in the start position"
    );
}

#[test]
fn best_move_tracks_the_position_it_was_given() {
    // Play the engine's own first move, then search again. The second result must
    // be legal in the new position, which it would not be if the accessor were
    // returning a stale value from the previous search.
    let pos = Position::startpos();
    let first = search_for_nodes(&pos, 20_000).expect("no best move for the start position");

    let next = pos.apply_move(first);
    let second = search_for_nodes(&next, 20_000).expect("no best move for the second position");

    let mut legal = Vec::new();
    movegen::generate_moves(&mut legal, &next);

    assert!(
        legal.contains(&second),
        "search returned {second}, which is not legal after {first}"
    );
}
