struct Bounds(usize, usize);

struct Point {
    x: u8,
    y: u8
}

#[test]
fn test() {
    let balloon = Point{
        x: 2,
        y: 1
    };
    match balloon {
        Point { x: 0, y: height } =>
            println!("straight up {} meters", height),
        Point { x: x, y: y } =>
            println!("at ({}m, {}m)", x, y),
    }
    
}