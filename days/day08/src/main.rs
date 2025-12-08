use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], debug: u8) -> impl Display + 'static {
        let FindPairs {
            pairs,
            mut sets,
            boxes: _boxes,
        } = find_pairs(input);

        let target_connections = match debug {
            0 => 1000,
            1 => 10,
            2 => 3,
            _ => panic!("unknown debug flag"),
        };

        for &(_d, i, j) in pairs.iter().take(target_connections) {
            sets.union(i, j);
        }

        let f: Vec<usize> = frequencies(sets);
        f[..3].iter().product::<usize>()
    }

    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static {
        let FindPairs {
            pairs,
            mut sets,
            boxes,
        } = find_pairs(input);

        let mut connections = 0;

        for &(_d, i, j) in pairs.iter() {
            if sets.union(i, j) {
                connections += 1;
                if connections == boxes.len() - 1 {
                    let b1 = boxes[i];
                    let b2 = boxes[j];
                    return b1[2] as u64 * b2[2] as u64;
                }
            }
        }
        panic!("ran out of pairs");
    }
}

struct FindPairs {
    pairs: Vec<(u64, usize, usize)>,
    sets: petgraph::unionfind::UnionFind<usize>,
    boxes: Vec<[u32; 3]>,
}

fn find_pairs(input: &[u8]) -> FindPairs {
    let boxes: Vec<[u32; 3]> = parse(input);
    let mut pairs = Vec::new();

    for (i, &b1) in boxes.iter().enumerate() {
        for (j, &b2) in boxes.iter().enumerate().skip(i + 1) {
            let d = distance(b1, b2);
            pairs.push((d, i, j));
        }
    }

    pairs.sort_unstable_by_key(|&(d, ..)| d);
    let sets = petgraph::unionfind::UnionFind::new(boxes.len());
    FindPairs { pairs, sets, boxes }
}

fn frequencies(sets: petgraph::unionfind::UnionFind<usize>) -> Vec<usize> {
    sets.into_labeling()
        .into_iter()
        .counts()
        .into_values()
        .sorted_unstable_by_key(|&i| Reverse(i))
        .collect()
}

fn distance(b1: [u32; 3], b2: [u32; 3]) -> u64 {
    let [z1, y1, x1] = b1.map(|n| n as i64);
    let [z2, y2, x2] = b2.map(|n| n as i64);
    ((z1 - z2).pow(2) + (y1 - y2).pow(2) + (x1 - x2).pow(2)) as u64
}

fn parse(input: &[u8]) -> Vec<[u32; 3]> {
    let mut boxes = Vec::new();
    let mut input = Consume::new(input);
    while !input.is_empty() {
        let x = input.int().unwrap();
        assert!(input.byte(b','));
        let y = input.int().unwrap();
        assert!(input.byte(b','));
        let z = input.int().unwrap();
        assert!(input.newline());
        boxes.push([z, y, x]);
    }
    boxes
}
