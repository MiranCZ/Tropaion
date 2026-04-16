use std::io::empty;
use tropaion::run_code_with_args;
use tropaion::util::arg_convertor::{into_arg, struct_convertor, ValueConvertable, ValueLike};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl ValueLike for Direction {
    fn into_convertable(self) -> ValueConvertable {
        match self {
            Self::Up => 99.into_convertable(),
            Self::Down => 88.into_convertable(),
            Self::Left => 77.into_convertable(),
            Self::Right => 66.into_convertable(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl ValueLike for Point {
    fn into_convertable(self) -> ValueConvertable {
        let mut s = struct_convertor("Point");
        s.add_field(self.x);
        s.add_field(self.y);
        s.convert()
    }
}

#[derive(Debug, Clone)]
pub struct Snake {
    points: Vec<Point>,
    direction: Direction,
}

impl ValueLike for Snake {
    fn into_convertable(self) -> ValueConvertable {
        let mut s = struct_convertor("Snake");
        s.add_field(self.direction);
        s.add_field(self.points);
        s.convert()
    }
}



#[test]
fn test_arg_passing() {
    let code = r#"
    struct Point(x: int, y: int);
    struct Snake(direction: Direction, points: Vec<Point>);

    fn main(snake: Snake, p: Point) -> int {
        return snake.direction.rot_clockwise();
    }
    "#;

    let snake = Snake{
        points: vec![Point{x: 1000, y:10007}, Point{x: 41, y: 42}],
        direction: Direction::Up
    };

    run_code_with_args(code.to_string(),"main", vec![
        into_arg(snake), into_arg(Point{x: 666, y: 777})
    ], &mut empty()).unwrap();
}


#[test]
fn test_arg_passing2() {
    let code = r#"
    struct Point(x: int, y: int);
    struct Snake(direction: Direction, points: Vec<Point>);

    fn main(snake: Snake) -> int {
        return snake.direction.rot_clockwise();
    }
    "#;

    let snake = Snake{
        points: vec![Point{x: 1000, y:10007}, Point{x: 41, y: 42}, Point{x: 666, y: 777}],
        direction: Direction::Up
    };

    run_code_with_args(code.to_string(),"main", vec![
        into_arg(snake),
    ], &mut empty()).unwrap();
}