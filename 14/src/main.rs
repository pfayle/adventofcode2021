use std::collections::HashMap;
use std::io;
use std::env;

const DEBUG: bool = false;

fn main() {
    let args: Vec<String> = env::args().collect();
    let runs = args[1].parse::<usize>().unwrap();

    let mut template = String::new();
    let mut rules: HashMap<(char, char), char> = HashMap::new();
    loop {
        let mut buf = String::new();
        let n = io::stdin().read_line(&mut buf);
        match n {
            Ok(0) => { break },
            Ok(_) => { parse_line(&buf.trim(), &mut template, &mut rules) },
            Err(_) => { break; },
        }
    }
    if DEBUG {
        println!("Template: {}; rules: {:?}", template, rules);
    }
    //let mut r: String = template.clone();
    //let mut charcountx: HashMap<char, i64> = HashMap::new();
    /* Part 1 function, too inefficient for part 2:
    for n in 0..runs {
        (r, charcountx) = process(&r, &rules);
        println!("Charcount: {:?}", charcountx);
        if DEBUG {
            println!("{}", n);
            println!("max-min: {}", find_difference(&charcountx));
        }
    }
    */
    let pairs = count_pairs(template.as_str(), &rules, runs);
    let chars = count_chars(pairs);
    if DEBUG {
        println!("{:?}", chars);
    }
    println!("new max-min: {}", find_difference(&chars));
}

fn parse_line(line: &str, template: &mut String, rules: &mut HashMap<(char, char), char>) {
    if line.contains('>') {
        let chars: Vec<char> = line.chars().collect();
        rules.insert((chars[0], chars[1]), chars[6]); // very cheap regex
    } else if line.len() > 0 {
        *template = line.to_string();
    }
}

fn process(line: &str, rules: &HashMap<(char, char), char>) -> (String, HashMap<char, i64>) {
    let chars = line.chars().collect::<Vec<char>>();
    let mut charcounts: HashMap<char, i64> = HashMap::new();
    let first = chars[0];
    charcounts.insert(first, 1);
    let mut r: Vec<char> = vec![first];
    let mut iter = chars.windows(2);
    while let Some(&[c1, c2]) = iter.next() {
        let newchar = *rules.get(&(c1, c2)).unwrap();
        r.push(newchar);
        r.push(c2);
        if charcounts.contains_key(&newchar) {
            *charcounts.get_mut(&newchar).unwrap() += 1;
        } else {
            charcounts.insert(newchar, 1);
        }
        if charcounts.contains_key(&c2) {
            *charcounts.get_mut(&c2).unwrap() += 1;
        } else {
            charcounts.insert(c2, 1);
        }
    }
    (r.iter().collect::<String>(), charcounts)
}

fn find_difference(charcount: &HashMap<char, i64>) -> i64 {
    charcount.values().max().unwrap() - charcount.values().min().unwrap()
}

fn count_pairs(s: &str, rules: &HashMap<(char, char), char>, steps: usize) -> HashMap<(char, char), i64> {
    let chars = s.chars().collect::<Vec<char>>();
    let mut iter = chars.windows(2);
    let mut pairs = HashMap::new();
    while let Some(&[c1, c2]) = iter.next() {
        if pairs.contains_key(&(c1, c2)) {
            *pairs.get_mut(&(c1, c2)).unwrap() += i64::from(1);
        } else {
            pairs.insert((c1, c2), i64::from(1));
        }
    }
    for _ in 0..steps {
        pairs = map_pairs(pairs, rules);
    }
    pairs 
}

fn map_pairs(pairs: HashMap<(char, char), i64>, rules: &HashMap<(char, char), char>) -> HashMap<(char, char), i64> {
    let mut r = HashMap::new();
    for ((a, b), v) in pairs {
        let newchar = *rules.get(&(a, b)).unwrap();
        if r.contains_key(&(a, newchar)) {
            *r.get_mut(&(a, newchar)).unwrap() += v;
        } else {
            r.insert((a, newchar), v);
        }
        if r.contains_key(&(newchar, b)) {
            *r.get_mut(&(newchar, b)).unwrap() += v;
        } else {
            r.insert((newchar, b), v);
        }
    }
    r
}

fn count_chars(pairs: HashMap<(char, char), i64>) -> HashMap<char, i64> {
    let mut r = HashMap::new();
    for ((_, k), v) in pairs {
        if r.contains_key(&k) {
            *r.get_mut(&k).unwrap() += v;
        } else {
            r.insert(k, v);
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_to_char_count() {
        {
            let mut pairs: HashMap<(char, char), i64> = HashMap::new();
            pairs.insert(('N', 'N'), 5);
            let r: HashMap<char, i64> = count_chars(pairs);
            assert_eq!(*r.get(&'N').unwrap(), i64::from(5));
        }
        {
            let mut pairs: HashMap<(char, char), i64> = HashMap::new();
            // NANANANANANANNNNNN
            // 6 As, 12 Ns, ignore initial N
            pairs.insert(('N', 'N'), 5);
            pairs.insert(('N', 'A'), 6);
            pairs.insert(('A', 'N'), 6);
            let r: HashMap<char, i64> = count_chars(pairs);
            assert_eq!(*r.get(&'N').unwrap(), i64::from(11));
            assert_eq!(*r.get(&'A').unwrap(), i64::from(6));
        }
    }

    #[test]
    fn test_map_pairs() {
        let mut pairs: HashMap<(char, char), i64> = HashMap::new();
        pairs.insert(('N', 'N'), 1);
        let mut rules: HashMap<(char, char), char> = HashMap::new();
        rules.insert(('N', 'N'), 'A');
        let r = map_pairs(pairs, &rules);
        assert_eq!(*r.get(&('N', 'A')).unwrap(), i64::from(1));
        assert_eq!(*r.get(&('A', 'N')).unwrap(), i64::from(1));
    }

    #[test]
    fn count_pairs_2len_0step() {
        let s = "NN";
        let rules: HashMap<(char, char), char> = HashMap::new();
        let pairs = count_pairs(s, &rules, 0);
        assert_eq!(*pairs.get(&('N', 'N')).unwrap(), i64::from(1));
    }

    #[test]
    fn count_pairs_2len_1step() {
        {
            let s = "NN";
            let mut rules: HashMap<(char, char), char> = HashMap::new();
            rules.insert(('N','N'), 'N');
            let pairs = count_pairs(s, &rules, 1);
            assert_eq!(*pairs.get(&('N', 'N')).unwrap(), i64::from(2));
        }
        {
            let s = "NN";
            let mut rules: HashMap<(char, char), char> = HashMap::new();
            rules.insert(('N','N'), 'A');
            let pairs = count_pairs(s, &rules, 1);
            assert_eq!(*pairs.get(&('N', 'A')).unwrap(), i64::from(1));
            assert_eq!(*pairs.get(&('A', 'N')).unwrap(), i64::from(1));
        }
    }

    #[test]
    fn count_pairs_2len_2step() {
        {
            let s = "NN";
            let mut rules: HashMap<(char, char), char> = HashMap::new();
            rules.insert(('N','N'), 'N');
            let pairs = count_pairs(s, &rules, 2);
            assert_eq!(*pairs.get(&('N', 'N')).unwrap(), i64::from(4));
        }
    }

    #[test]
    fn count_pairs_4len_10step() {
        {
            let s = "NNNN";
            let mut rules: HashMap<(char, char), char> = HashMap::new();
            rules.insert(('N','N'), 'N');
            let pairs = count_pairs(s, &rules, 10);
            assert_eq!(*pairs.get(&('N', 'N')).unwrap(), i64::from(3072));
        }
    }

    #[test]
    fn count_pairs_2len_40step() {
        {
            let s = "NN";
            let mut rules: HashMap<(char, char), char> = HashMap::new();
            rules.insert(('N','N'), 'N');
            let pairs = count_pairs(s, &rules, 40);
            assert_eq!(*pairs.get(&('N', 'N')).unwrap(), 1099511627776);
        }
    }
}