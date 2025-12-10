#![allow(unused)]

use std::collections::HashSet;
use std::io::{Write, stderr};
use std::ops::ControlFlow;

use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let tiles = parse_tiles(input);
        tile_pairs(&tiles).map(|[a, b]| area(a, b)).max().unwrap()
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let mut tiles = parse_tiles(input);
        tiles.push(tiles[0]);
        let mut colored_tiles: HashSet<[u32; 2]> = HashSet::with_capacity(100000);

        eprintln!("colored tiles");
        for &[[ax, ay], [bx, by]] in tiles.array_windows() {
            #[allow(clippy::collapsible_else_if)]
            if ax == bx {
                if ay < by {
                    colored_tiles.extend((ay..=by).map(move |y| [ax, y]))
                } else {
                    colored_tiles.extend((by..=ay).map(move |y| [ax, y]))
                }
            } else {
                if ax < bx {
                    colored_tiles.extend((ax..=bx).map(move |x| [x, ay]))
                } else {
                    colored_tiles.extend((bx..=ax).map(move |x| [x, ay]))
                }
            }
        }

        let mut inside_tiles = HashSet::with_capacity(100000);
        let mut outside_tiles = HashSet::with_capacity(100000);

        eprintln!("inside and outside tiles");

        for &[[ax, ay], [bx, by]] in tiles.array_windows() {
            #[allow(clippy::collapsible_else_if)]
            if ax == bx {
                if ay < by {
                    inside_tiles.extend((ay..by + 1).map(|y| [ax + 1, y]));
                    outside_tiles.extend((ay..by + 1).map(|y| [ax - 1, y]));
                } else {
                    inside_tiles.extend((by..ay + 1).map(|y| [ax - 1, y]));
                    outside_tiles.extend((by..ay + 1).map(|y| [ax + 1, y]));
                }
            } else {
                if ax < bx {
                    inside_tiles.extend((ax..bx + 1).map(|x| [x, ay - 1]));
                    outside_tiles.extend((ax..bx + 1).map(|x| [x, ay + 1]));
                } else {
                    inside_tiles.extend((bx..ax + 1).map(|x| [x, ay + 1]));
                    outside_tiles.extend((bx..ax + 1).map(|x| [x, ay - 1]));
                }
            }
        }

        inside_tiles.retain(|p| !colored_tiles.contains(p));
        outside_tiles.retain(|p| !colored_tiles.contains(p));

        // print_points([
        //     (&outside_tiles, 'O'),
        //     (&inside_tiles, 'I'),
        //     (&colored_tiles, '#'),
        // ]);

        if inside_tiles.len() > outside_tiles.len() {
            swap(&mut inside_tiles, &mut outside_tiles);
        }

        // eprintln!(
        //     "{:?}",
        //     colored_tiles
        //         .iter()
        //         .copied()
        //         .sorted_unstable()
        //         .collect_vec()
        // );

        eprintln!("sort pairs");

        let mut pairs: BinaryHeap<_> = tile_pairs(&tiles)
            .map(|[a, b]| {
                let area = area(a, b);
                (area, a, b)
            })
            .collect();

        eprintln!("find best rectangle | rectangles: {}", pairs.len());

        'a: for (i, (area, a, b)) in pairs.into_iter_sorted().enumerate() {
            if i.is_multiple_of(1000) {
                eprintln!("{i}");
            }
            stderr().flush().unwrap();
            if perimeter(a, b, |p| !outside_tiles.contains(&p)) {
                return area;
            }
        }

        panic!("no rectangles found");
    }
}

fn print_points<const N: usize>(pairs: [(&HashSet<[u32; 2]>, char); N]) {
    let (min_x, max_x) = pairs
        .into_iter()
        .flat_map(|(points, _)| points)
        .map(|&[x, _]| x)
        .minmax()
        .into_option()
        .unwrap();
    let (min_y, max_y) = pairs
        .into_iter()
        .flat_map(|(points, _)| points)
        .map(|&[_, y]| y)
        .minmax()
        .into_option()
        .unwrap();

    // eprintln!("{min_x} {max_x}, {min_y} {max_y}");

    for y in min_y..=max_y {
        'a: for x in min_x..=max_x {
            for (points, c) in pairs {
                if points.contains(&[x, y]) {
                    eprint!("{}", c);
                    continue 'a;
                }
            }
            eprint!(".");
        }
        eprintln!();
    }
    eprintln!();
}

fn area([ax, ay]: [u32; 2], [bx, by]: [u32; 2]) -> u64 {
    (ax.abs_diff(bx) + 1) as u64 * (ay.abs_diff(by) + 1) as u64
}

fn perimeter(a: [u32; 2], b: [u32; 2], mut f: impl FnMut([u32; 2]) -> bool) -> bool {
    let [ax, ay] = a;
    let [bx, by] = b;
    let horizontal = if ax < bx { ax..bx + 1 } else { bx..ax + 1 };
    let vertical = if ay < by { ay..by + 1 } else { by..ay + 1 };

    for p in horizontal
        .flat_map(|x| [[x, ay], [x, by]])
        .chain(vertical.flat_map(|y| [[ax, y], [bx, y]]))
    {
        if !f(p) {
            return false;
        }
    }
    true
}

fn tile_pairs(tiles: &[[u32; 2]]) -> impl Iterator<Item = [[u32; 2]; 2]> {
    tiles
        .iter()
        .enumerate()
        .flat_map(|(i, &a)| tiles.iter().skip(i + 1).map(move |&b| [a, b]))
}

fn parse_tiles(input: &[u8]) -> Vec<[u32; 2]> {
    let mut tiles = Vec::new();
    let mut input = Consume::new(input);

    while !input.is_empty() {
        let x = input.int().unwrap();
        assert!(input.byte(b','));
        let y = input.int().unwrap();
        assert!(input.newline());
        tiles.push([x, y]);
    }

    tiles
}
