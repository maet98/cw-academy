fn main() {
    let mut a = 0;
    let mut b = 1;
    let mut n = 10;
    if n == 1 {
        println!("1");
    }
    n -= 1;
    while n > 0 {
        println!("{} {}", a, b);
        let temp = b;
        b = b + a;
        a = temp;
        n -= 1
    }
    println!("hi {}", b);
}
