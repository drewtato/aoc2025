use helpers::*;

fn main() {
    use solver_interface::ChildSolverExt;
    Solver::run().unwrap_display();
}

struct Solver;

impl solver_interface::ChildSolver for Solver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static {
        let (shapes, trees) = read_shapes_and_trees(input);
        // eprintln!("{:?}", shapes);
        // eprintln!("{:?}", trees);
        let shape_areas = shapes.iter().map(|shape| shape.area()).collect_vec();
        let mut valid = 0;
        for tree in trees {
            let area = tree.area();
            let filled: usize = tree
                .shapes
                .iter()
                .zip(&shape_areas)
                .map(|(&count, &area)| count * area)
                .sum();
            if area > filled {
                valid += 1;
            }
        }
        valid
    }

    fn part_two(_input: &[u8], _debug: u8) -> impl Display + 'static {
        "woohoo"
    }
}

#[derive(Clone, Copy)]
struct Shape([bool; 9]);

impl Debug for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for line in self.0.chunks(3) {
            for &x in line {
                write!(f, "{}", if x { '#' } else { '.' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Shape {
    fn new(shape: [bool; 9]) -> Self {
        Self(shape)
    }

    fn from_consume(c: &mut Consume) -> Result<Self, Option<u8>> {
        let mut shape = [false; 9];
        for row in shape.chunks_mut(3) {
            for x in row {
                match c.consume_byte() {
                    Some(b'#') => *x = true,
                    Some(b'.') => *x = false,
                    b => return Err(b),
                }
            }
            assert!(c.newline());
        }
        Ok(Self::new(shape))
    }

    fn area(&self) -> usize {
        self.0.iter().filter(|&&b| b).count()
    }
}

const MAX_SHAPES: usize = 6;

#[derive(Debug, Clone)]
struct Tree {
    width: usize,
    length: usize,
    shapes: ArrayVec<usize, MAX_SHAPES>,
}

impl Tree {
    fn new<E>(
        width: usize,
        length: usize,
        shapes: impl IntoIterator<Item = Result<usize, E>>,
    ) -> Result<Self, E> {
        let shapes = shapes.into_iter().collect::<Result<_, _>>()?;
        Ok(Self {
            length,
            width,
            shapes,
        })
    }

    fn from_consume(c: &mut Consume) -> Result<Self, Option<u8>> {
        (|| {
            let width = c.int()?;
            c.byte(b'x').then_some(())?;
            let length = c.int()?;
            c.byte(b':').then_some(())?;
            let shapes = fn_iter(|| c.byte(b' ').then(|| c.int().ok_or(())));
            Tree::new(width, length, shapes).ok()
        })()
        .ok_or_else(|| c.consume_byte())
    }

    fn area(&self) -> usize {
        self.length * self.width
    }
}

fn read_shapes_and_trees(input: &[u8]) -> (Vec<Shape>, Vec<Tree>) {
    let mut input = Consume::new(input);
    let mut shapes = Vec::new();
    let before = loop {
        let before = input.slice();
        let index: usize = input.int().unwrap();
        if !input.byte(b':') {
            break before;
        }
        assert_eq!(index, shapes.len());
        input.assert_byte(b'\n');
        let shape = Shape::from_consume(&mut input).unwrap();
        shapes.push(shape);
        input.assert_byte(b'\n');
    };

    let mut input = Consume::new(before);
    let mut trees = Vec::new();
    while !input.is_empty() {
        trees.push(Tree::from_consume(&mut input).unwrap());
        input.assert_byte(b'\n');
    }

    (shapes, trees)
}
