use std::io;
use itertools::Itertools;

const UNIQUE_LENGTHS: [usize; 4] = [2,3,4,7];
fn main() -> io::Result<()> {
    let mut lines: Vec<String> = vec![];
    loop {
        let mut buf = String::new();
        let n = io::stdin().read_line(&mut buf);
        match n {
            Ok(0) => { break },
            Ok(_) => { lines.push(String::from(buf.trim())) },
            Err(_) => { break; },
        }
    }

    let count: usize = lines.iter().map(|x| count_unique_lengths(x)).sum();
    println!("Number of 1/4/7/8s: {}", count);

    let count2: i32 = lines.iter().map(|x| line_to_num(x)).sum();
    println!("Sum of all values: {}", count2);

    Ok(())
}

fn count_unique_lengths(s: &str) -> usize {
    let ss: Vec<&str> = s.split("|").collect();
    let r = ss[1]
        .split(" ")
        .map(|x| x.len())
        .filter(|x| UNIQUE_LENGTHS.contains(x))
        .count();
    r
}

fn string_to_digit(s: &str, code_map: &[String; 10]) -> char {
    let o_s: String = s.chars().sorted().collect();
    char::from_digit(code_map.iter().position(|x| *x == o_s).unwrap() as u32, 10).unwrap()
}

fn line_to_num(s: &str) -> i32 {
    let mut split = s.split("|");
    let (input, output) = split.next_tuple().unwrap();
    let code_map = process_input(
        &input.split(" ")
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
    );
    output.trim().split(" ").map(|x| string_to_digit(x, &code_map))
    .collect::<String>().parse::<i32>().unwrap()
}

fn process_input<'a>(input: &Vec<&'a str>) -> [String; 10] {
    let mut r: [String; 10] = Default::default();
    // order input strings first.
    let o_input: Vec<String> = input.iter().map(|&s| s.chars().sorted().collect::<String>()).collect();
    for item in &o_input {
        let index = match item.chars().count() {
            2 => 1,
            3 => 7,
            4 => 4,
            7 => 8,
            _ => 0,
        };
        r[index] = item.clone();
    }
    for item in &o_input {
        if item.len() != 6 {
            continue;
        }
        if string_difference(&item, &r[1]) == 5 {
            r[6] = item.clone();
        } else if string_difference(&item, &r[4]) == 3 {
            r[0] = item.clone();
        } else {
            r[9] = item.clone();
        }
    }
    for item in &o_input {
        if item.len() != 5 {
            continue;
        }
        if string_difference(&r[1], &item) == 0 {
            r[3] = item.clone();
        } else if string_difference(&r[4], &item) == 1 {
            r[5] = item.clone();
        } else {
            r[2] = item.clone();
        }
    }
    r
}

fn string_difference(a: &str, b: &str) -> usize {
    a.chars().filter(|c| !b.contains(*c)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1478() {
        let input = vec![
            "ab",
            "abcd",
            "abe",
            "abcdefg",
        ];
        let r = process_input(&input);
        assert_eq!(r[1], "ab");
        assert_eq!(r[4], "abcd");
        assert_eq!(r[7], "abe");
        assert_eq!(r[8], "abcdefg");
    }

    #[test]
    fn test_len_5() {
        let input = vec![
            "abcefg", //0 abcefg
            "cf", //1 cf
            "acdeg", //2 acdeg
            "acdfg", //3 acdfg
            "bcdf", //4 bcdf
            "abdfg", //5 abdfg
            "abdefg", //6 abdefg
            "acf", //7 acf
            "abcdefg", //8
            "abcdfg", //9 abcdfg
        ];
        let r = process_input(&input);
        assert_eq!(r[2], "acdeg");
        assert_eq!(r[5], "abdfg");
        assert_eq!(r[3], "acdfg");
    }

    #[test]
    fn test_len_6() {
        let input = vec![
            "abcefg", //0 abcefg
            "cf", //1 cf
            "acdeg", //2 acdeg
            "acdfg", //3 acdfg
            "bcdf", //4 bcdf
            "abdfg", //5 abdfg
            "abdefg", //6 abdefg
            "acf", //7 acf
            "abcdefg", //8
            "abcdfg", //9 abcdfg
        ];
        let r = process_input(&input);
        assert_eq!(r[0], "abcefg");
        assert_eq!(r[6], "abdefg");
        assert_eq!(r[9], "abcdfg");
    }

    #[test]
    fn test_unordered() {
        let input = vec![
            "gabcef", //0 abcefg
            "fc", //1 cf
            "gacde", //2 acdeg
            "gacdf", //3 acdfg
            "bcdf", //4 bcdf
            "gabdf", //5 abdfg
            "gabdef", //6 abdefg
            "acf", //7 acf
            "gabcdef", //8
            "gabcdf", //9 abcdfg
        ];
        let r = process_input(&input);
        assert_eq!(r[0], "abcefg");
    }
}