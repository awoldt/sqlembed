mod utils;

fn main() {
    let file_string = utils::read_pdf("./17029.pdf");
    if !file_string.is_ok() {
        println!("{:?}", file_string.err());
        return;
    } else {
        let text = file_string.unwrap();
        println!("results");
        println!("{}", text);
    }
}
