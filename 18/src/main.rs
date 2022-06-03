use std::{fmt::Display, ops::Add};
use std::{io, io::Read};
use itertools::Itertools;

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Error reading stdin.");
    let r = buf.trim().lines().into_iter()
        .map(SnailfishNum::from)
        .reduce(|a, s| (a+s).full_reduce())
        .unwrap().magnitude();
    println!("Final sum magnitude: {}", r);

    let iter = buf.trim().lines().into_iter()
        .map(SnailfishNum::from)
        .combinations(2);
    let max = iter.flat_map(|v| vec![
            (v[0].clone()+v[1].clone()).full_reduce().magnitude(),
            (v[1].clone()+v[0].clone()).full_reduce().magnitude()
        ])
        .max().unwrap();
    println!("Maximum sum: {}", max);
}

#[derive(PartialEq, Clone, Debug)]
enum Term {
    Atom(usize),
    Tuple(SnailfishNum),
}

impl Term {
    fn magnitude(&self) -> usize {
        match self {
            Term::Atom(a) => *a,
            Term::Tuple(s) => s.magnitude(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
enum Lex {
    Term(Term),
    Comma,
    OpenBracket,
    CloseBracket,
}

#[derive(PartialEq, Clone, Debug)]
struct SnailfishNum(Box<Term>, Box<Term>);

impl Display for SnailfishNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::from("[");
        r += &match &*self.0 {
            Term::Atom(a) => a.to_string(),
            Term::Tuple(t) => format!("{}", t),
        };
        r += ",";
        r += &match &*self.1 {
            Term::Atom(a) => a.to_string(),
            Term::Tuple(t) => format!("{}", t),
        };
        r += "]";
        write!(f, "{}", r)
    }
}

impl Add for SnailfishNum {
    type Output = SnailfishNum;

    fn add(self, rhs: Self) -> Self::Output {
        SnailfishNum(Box::new(Term::Tuple(self)), Box::new(Term::Tuple(rhs)))
    }
}

impl From<Vec<Lex>> for SnailfishNum {
    fn from(mut v: Vec<Lex>) -> Self {
        while v.contains(&Lex::Comma) {
            let mut iter = v.iter().peekable();
            let mut v2 = vec![];
            while let Some(l) = iter.next() {
                match l {
                    Lex::OpenBracket => {
                        if let Some(Lex::Term(lhs)) = iter.peek() {
                            iter.next();
                            if let Some(Lex::Comma) = iter.peek() {
                                iter.next();
                                if let Some(Lex::Term(rhs)) = iter.peek() {
                                    iter.next();
                                    if let Some(Lex::CloseBracket) = iter.peek() {
                                        iter.next();
                                        v2.push(Lex::Term(Term::Tuple(SnailfishNum(Box::new(lhs.clone()), Box::new(rhs.clone())))));
                                    } else {
                                        v2.push(l.clone());
                                        v2.push(Lex::Term(lhs.clone()));
                                        v2.push(Lex::Comma);
                                        v2.push(Lex::Term(rhs.clone()));
                                    }
                                } else {
                                    v2.push(l.clone());
                                    v2.push(Lex::Term(lhs.clone()));
                                    v2.push(Lex::Comma);
                                }
                            } else {
                                v2.push(l.clone());
                                v2.push(Lex::Term(lhs.clone()));
                            }
                        } else {
                            v2.push(l.clone());
                        }
                    },
                    _ => { v2.push(l.clone()); },
                }
            }
            if v == v2 {
                println!("{:?}", v);
                panic!();
            }
            v = v2;
        }
        if let Lex::Term(Term::Tuple(s)) = v[0].clone() {
            s
        } else {
            unreachable!()
        }
    }
}

impl From<&str> for SnailfishNum {
    fn from(s: &str) -> Self {
        Self::lex(String::from(s)).into()
    }
}

impl SnailfishNum {
    fn lex(s: String) -> Vec<Lex> {
        let mut iter = s.chars().into_iter().peekable();
        let mut v: Vec<Lex> = vec![];
        while let Some(c) = iter.next() {
            v.push(match c {
                '[' => Lex::OpenBracket,
                '0'..='9' => {
                    let mut r = String::from(c);
                    while let Some(c2) = iter.peek() {
                        if c2.is_numeric() {
                            r.push(*c2);
                            iter.next();
                        } else {
                            break;
                        }
                    }
                    Lex::Term(Term::Atom(r.parse::<usize>().unwrap()))
                },
                ']' => Lex::CloseBracket,
                ',' => Lex::Comma,
                _ => unreachable!(),
            });
        }
        v
    }

    fn explode(&self) -> Self {
        let lex = Self::lex(self.to_string());
        let mut r: Vec<Lex> = vec![];
        let mut left: Option<Lex> = None;
        let mut level = 0;
        let mut iter = lex.iter();
        let mut cache: Vec<Lex> = vec![];
        while let Some(c) = iter.next() {
            match c {
                Lex::OpenBracket => {
                    level += 1;
                    if level == 5 {
                        let left_new = iter.next().unwrap();
                        iter.next();
                        let right_new = iter.next().unwrap();
                        iter.next();
                        if let Some(Lex::Term(Term::Atom(mut l_val))) = left {
                            if let Lex::Term(Term::Atom(l_val2)) = left_new {
                                l_val += l_val2;
                                r.push(Lex::Term(Term::Atom(l_val)));
                            }
                        }
                        r.append(&mut cache);
                        r.push(Lex::Term(Term::Atom(0)));
                        left = None;
                        for c2 in iter.by_ref() {
                            if let Lex::Term(Term::Atom(r_val)) = c2 {
                                if let Lex::Term(Term::Atom(r_val2)) = right_new {
                                    r.push(Lex::Term(Term::Atom(r_val + r_val2)));
                                    break;
                                } else {
                                    r.push(c2.clone());
                                }
                            } else {
                                r.push(c2.clone());
                            }
                        }
                        for c2 in iter.by_ref() {
                            r.push(c2.clone());
                        }
                    } else {
                        cache.push(c.clone());
                    }
                },
                Lex::CloseBracket => {
                    level -= 1;
                    cache.push(c.clone());
                },
                Lex::Term(_) => {
                    if let Some(c2) = left {
                        r.push(c2.clone());
                    }
                    r.append(&mut cache);
                    left = Some(c.clone());
                },
                Lex::Comma => {cache.push(c.clone())},
            }
        }
        if let Some(c2) = left {
            r.push(c2);
        }
        r.append(&mut cache);
        r.into()
    }

    fn split(&self) -> Self {
        let s = Self::lex(self.to_string());
        let mut iter = s.iter();
        let mut r: Vec<Lex> = vec![];
        while let Some(lex) = iter.next() {
            if let Lex::Term(Term::Atom(val)) = lex {
                if val >= &10 {
                    r.push(Lex::OpenBracket);
                    r.push(Lex::Term(Term::Atom((val)/2)));
                    r.push(Lex::Comma);
                    r.push(Lex::Term(Term::Atom((val+1)/2)));
                    r.push(Lex::CloseBracket);
                    r.append(&mut iter.cloned().collect());
                    break;
                }
            }
            r.push(lex.clone());
        }
        r.into()
    }

    fn reduce(&self) -> Self {
        let e = self.explode();
        if e == *self {
            self.split()
        } else {
            e
        }
    }

    fn full_reduce(&self) -> Self {
        let mut s = self.clone();
        loop {
            let s2 = s.reduce();
            if s2 == s {
                break;
            }
            s = s2;
        }
        s
    }

    fn magnitude(&self) -> usize {
        3*self.0.magnitude() + 2*self.1.magnitude()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start() {
        let s = SnailfishNum(Box::new(Term::Atom(1)), Box::new(Term::Atom(2)));
        if let Term::Atom(a) = *s.0 {
            assert_eq!(a, 1);
        }
        let s2 = SnailfishNum(Box::new(Term::Tuple(s)), Box::new(Term::Atom(3)));
        if let Term::Tuple(s3) = *s2.0 {
            if let Term::Atom(a) = *s3.1 {
                assert_eq!(a, 2);
            }
        }
    }

    #[test]
    fn display() {
        let s = SnailfishNum(Box::new(Term::Atom(1)), Box::new(Term::Atom(2)));
        assert_eq!(s.to_string(), "[1,2]");
        let s2 = SnailfishNum(Box::new(Term::Tuple(s)), Box::new(Term::Atom(3)));
        assert_eq!(s2.to_string(), "[[1,2],3]");
    }

    #[test]
    fn add() {
        let s = SnailfishNum(Box::new(Term::Atom(1)), Box::new(Term::Atom(2)));
        let s2 = SnailfishNum(Box::new(Term::Atom(1)), Box::new(Term::Atom(2)));
        assert_eq!((s+s2).to_string(), "[[1,2],[1,2]]");
    }

    #[test]
    fn from_string() {
        {
            let st = "[[1,2],3]";
            let s: SnailfishNum = st.into();
            assert_eq!(s.to_string(), st);
        }
        {
            let st = "[[[[1,3],[5,3]],[[1,3],[8,7]]],[[[4,9],[6,9]],[[8,2],[7,3]]]]";
            let s: SnailfishNum = st.into();
            assert_eq!(s.to_string(), st);
        }
        {
            let st = "[[6,[5,[4,[3,2]]]],1]";
            let s: SnailfishNum = st.into();
            assert_eq!(s.to_string(), st);
        }
        {
            let st = "[[[[0,7],4],[15,[0,13]]],[1,1]]";
            let s: SnailfishNum = st.into();
            assert_eq!(s.to_string(), st);
        }
    }

    #[test]
    fn lex() {
        {
            let s = "[[[[0,7],4],[15,[0,13]]],[1,1]]".to_string();
            assert_eq!(SnailfishNum::lex(s), vec![
                Lex::OpenBracket,
                Lex::OpenBracket,
                Lex::OpenBracket,
                Lex::OpenBracket,
                Lex::Term(Term::Atom(0)),
                Lex::Comma,
                Lex::Term(Term::Atom(7)),
                Lex::CloseBracket,
                Lex::Comma,
                Lex::Term(Term::Atom(4)),
                Lex::CloseBracket,
                Lex::Comma,
                Lex::OpenBracket,
                Lex::Term(Term::Atom(15)),
                Lex::Comma,
                Lex::OpenBracket,
                Lex::Term(Term::Atom(0)),
                Lex::Comma,
                Lex::Term(Term::Atom(13)),
                Lex::CloseBracket,
                Lex::CloseBracket,
                Lex::CloseBracket,
                Lex::Comma,
                Lex::OpenBracket,
                Lex::Term(Term::Atom(1)),
                Lex::Comma,
                Lex::Term(Term::Atom(1)),
                Lex::CloseBracket,
                Lex::CloseBracket,
            ]);
        }
    }

    #[test]
    fn explode() {
        {
            let s: SnailfishNum = "[[[[[1,1],2],3],4],5]".into();
            assert_eq!(s.explode().to_string(), "[[[[0,3],3],4],5]");
        }
        {
            let s: SnailfishNum = "[[6,[5,[4,[3,2]]]],1]".into();
            assert_eq!(s.explode().to_string(), "[[6,[5,[7,0]]],3]");
        }
        {
            let s: SnailfishNum = "[[[[0,7],4],[15,[0,13]]],[1,1]]".into();
            assert_eq!(s.explode().to_string(), s.to_string());
        }
    }

    #[test]
    fn split() {
        {
            let s: SnailfishNum = "[[[[0,7],4],[15,[0,13]]],[1,1]]".into();
            assert_eq!(s.split().to_string(), "[[[[0,7],4],[[7,8],[0,13]]],[1,1]]");
        }
        {
            let s: SnailfishNum = "[[[[0,7],4],[15,[0,13]]],[1,1]]".into();
            assert_eq!(s.split().to_string(), "[[[[0,7],4],[[7,8],[0,13]]],[1,1]]");
        }

    }

    #[test]
    fn reduce() {
        {
            let mut s: SnailfishNum = "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]".into();
            s = s.reduce();
            assert_eq!(s.to_string(), "[[[[0,7],4],[7,[[8,4],9]]],[1,1]]");
            s = s.reduce();
            assert_eq!(s.to_string(), "[[[[0,7],4],[15,[0,13]]],[1,1]]");
            s = s.reduce();
            assert_eq!(s.to_string(), "[[[[0,7],4],[[7,8],[0,13]]],[1,1]]");
            s = s.reduce();
            assert_eq!(s.to_string(), "[[[[0,7],4],[[7,8],[0,[6,7]]]],[1,1]]");
            s = s.reduce();
            assert_eq!(s.to_string(), "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
            s = s.reduce();
            assert_eq!(s.to_string(), "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
        }
        {
            let s: SnailfishNum = "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]".into();
            assert_eq!(s.full_reduce().to_string(), "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
        }
    }

    #[test]
    fn final_sum() {
        {
            let v = vec![
                "[1,1]",
                "[2,2]",
                "[3,3]",
                "[4,4]",
            ];
            assert_eq!(
                v.iter().map(|s| SnailfishNum::from(*s)).reduce(|a, s| (a+s).full_reduce()).unwrap().full_reduce().to_string(),
                "[[[[1,1],[2,2]],[3,3]],[4,4]]",
            );
        }
        {
            let v = vec![
                "[1,1]",
                "[2,2]",
                "[3,3]",
                "[4,4]",
                "[5,5]",
            ];
            assert_eq!(
                v.iter().map(|s| SnailfishNum::from(*s)).reduce(|a, s| (a+s).full_reduce()).unwrap().full_reduce().to_string(),
                "[[[[3,0],[5,3]],[4,4]],[5,5]]",
            );
        }
        {
            let v = vec![
                "[1,1]",
                "[2,2]",
                "[3,3]",
                "[4,4]",
                "[5,5]",
                "[6,6]",
            ];
            assert_eq!(
                v.iter().map(|s| SnailfishNum::from(*s)).reduce(|a, s| (a+s).full_reduce()).unwrap().full_reduce().to_string(),
                "[[[[5,0],[7,4]],[5,5]],[6,6]]",
            );
        }
        {
            let v = vec![
                "[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]",
                "[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]",
                "[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]",
                "[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]",
                "[7,[5,[[3,8],[1,4]]]]",
                "[[2,[2,2]],[8,[8,1]]]",
                "[2,9]",
                "[1,[[[9,3],9],[[9,0],[0,7]]]]",
                "[[[5,[7,4]],7],1]",
                "[[[[4,2],2],6],[8,7]]",
            ];
            assert_eq!(
                v.iter().map(|s| SnailfishNum::from(*s)).reduce(|a, s| (a+s).full_reduce()).unwrap().full_reduce().to_string(),
                "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]",
            );
        }
    }

    #[test]
    fn magnitude() {
        assert_eq!(Term::Atom(2).magnitude(), 2);
        assert_eq!(SnailfishNum::from("[2,3]").magnitude(), 12);
        assert_eq!(SnailfishNum::from("[2,[3,5]]").magnitude(), 3*2+2*(3*3+2*5));
        assert_eq!(SnailfishNum::from("[9,1]").magnitude(), 29);
        assert_eq!(SnailfishNum::from("[1,9]").magnitude(), 21);
        assert_eq!(SnailfishNum::from("[[9,1],[1,9]]").magnitude(), 129);
        assert_eq!(SnailfishNum::from("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]").magnitude(), 3488);
    }
}
