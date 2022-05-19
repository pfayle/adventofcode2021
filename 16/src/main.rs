use std::io;
use to_binary::BinaryString;

const DEBUG: bool = false;

fn main() -> io::Result<()> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let mut raw_bits = BinaryString::from_hex(&buf.trim()).unwrap().0.chars().into_iter().map(|c| c != '0').collect::<Vec<bool>>();
    let packet = Packet::parse(&mut raw_bits).unwrap();
    println!("Sum of versions: {}", packet.add_versions());
    println!("Value: {}", packet.evaluate());
    Ok(())
}

struct Packet {
    version: i32,
    type_id: i32,
    child: Type,
}

struct Literal {
    raw: Vec<bool>,
}

impl Literal {
    fn value(self: &Self) -> i64 {
        let mut r = 0;
        for &bit in &self.raw {
            r = 2*r + (bit as i64);
        }
        r
    }

    fn parse(bits: &mut Vec<bool>) -> Option<Literal> {
        if DEBUG{
            println!("Literal::parse: {:?}", bits);
        }
        let mut newbits = bits.clone();
        newbits.reverse();
        let mut v = vec![];
        loop {
            let end = match newbits.pop() {
                Some(x) => { !x },
                None => {
                    return None;
                },
            };
            for _ in 0..4 {
                match newbits.pop() {
                    Some(bit) => { v.push(bit); },
                    None => {
                        return None;
                    },
                }
            }
            if end {
                break;
            }
        }
        newbits.reverse();
        *bits = newbits;
        Some(Literal::from(v))
    }
}

impl From<Vec<bool>> for Literal {
    fn from(raw_bits: Vec<bool>) -> Self {
        let raw = raw_bits.clone();
        Self { raw }
    }
}

impl Packet {
    fn parse(raw_bits: &mut Vec<bool>) -> Option<Self> {
        if DEBUG {
            println!("Packet::parse {:?}", raw_bits);
        }
        let mut newbits = raw_bits.clone();
        newbits.reverse();
        //get version
        let mut version = 0;
        for _ in 0..3 {
            if let Some(b) = newbits.pop() {
                version = 2*version + (b as i32);
            } else {
                return None;
            }
        }
        //get type id
        let mut type_id = 0;
        for _ in 0..3 {
            if let Some(b) = newbits.pop() {
                type_id = 2*type_id + (b as i32);
            } else {
                return None;
            }
        }
        //delegate to literal or operator parse
        newbits.reverse();
        let child = if type_id == 4 {
            Type::Literal(Literal::parse(&mut newbits).unwrap())
        } else {
            Type::Operator(Operator::parse(&mut newbits).unwrap())
        };
        *raw_bits = newbits;
        Some(Self{
            version,
            type_id,
            child,
        })
    }

    fn add_versions(self: &Self) -> i32 {
        let mut r: i32 = 0;
        r += self.version;
        match &self.child {
            Type::Literal(_) => {},
            Type::Operator(op) => {
                r += op.add_versions();
            },
        }
        r
    }

    fn evaluate(self: &Self) -> i64 {
        match &self.child {
            Type::Literal(lit) => {
                lit.value()
            },
            Type::Operator(op) => {
                let mut iter = op.children.iter().map(|p| p.evaluate());
                match self.type_id {
                    0 => {
                        iter.fold(0, |acc, x| acc + x)
                    },
                    1 => {
                        iter.fold(1, |acc, x| acc * x)
                    },
                    2 => {
                        iter.min().unwrap()
                    },
                    3 => {
                        iter.max().unwrap()
                    },
                    5 => {
                        let (a, b) = (iter.next().unwrap(), iter.next().unwrap());
                        (a > b) as i64
                    },
                    6 => {
                        let (a, b) = (iter.next().unwrap(), iter.next().unwrap());
                        (a < b) as i64
                    },
                    7 => {
                        let (a, b) = (iter.next().unwrap(), iter.next().unwrap());
                        (a == b) as i64
                    },
                    _ => {
                        0
                    }
                }
            }
        }
    }
}

enum Type {
    Literal(Literal),
    Operator(Operator),
}

struct Operator {
    children: Vec<Packet>,
}

impl Operator {
    fn parse(raw_bits: &mut Vec<bool>) -> Option<Self> {
        if DEBUG {
            println!("Operator::parse {:?}", raw_bits);
        }
        let mut newbits = raw_bits.clone();
        newbits.reverse();
        let mut v = vec![];
        let length_type_id = match newbits.pop() {
            Some(b) => b,
            None => { return None; }
        };
        if length_type_id {
            let mut n = 0;
            for _ in 0..11 {
                if let Some(b) = newbits.pop() {
                    n = 2*n + (b as i64);
                } else {
                    return None;
                }
            }
            newbits.reverse();
            let mut children = vec![];
            for _ in 0..n {
                children.push(Packet::parse(&mut newbits).unwrap());
            }
            *raw_bits = newbits;
            Some(Self{
                children,
            })
        } else {
            let mut n = 0;
            for _ in 0..15 {
                if let Some(b) = newbits.pop() {
                    n = 2*n + (b as i64);
                } else {
                    return None;
                }
            }
            for _ in 0..n {
                if let Some(b) = newbits.pop() {
                    v.push(b);
                } else {
                    return None;
                }
            }
            let mut children = vec![];
            newbits.reverse();
            while let Some(packet) = Packet::parse(&mut v) {
                children.push(packet);
            }
            *raw_bits = newbits;
            Some(Self{
                children,
            })
        }
    }

    fn add_versions(self: &Self) -> i32 {
        let mut r = 0;
        for c in &self.children {
            r += c.add_versions();
        }
        r
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    fn version2() -> Vec<bool> {
        vec![false, true, false]
    }

    fn type4() -> Vec<bool> {
        vec![true, false, false]
    }

    fn type5() -> Vec<bool> {
        vec![true, false, true]
    }

    fn typen(n: i32) -> Vec<bool> {
        vec![n>=4, n % 4 >= 2, n % 2 == 1]
    }

    fn length_type_id_0_11() -> Vec<bool> {
        vec![
            false,
            false, false, false, false, false,
            false, false, false, false, false,
            false, true, false, true, true,
        ]
    }

    fn length_type_id_1_1() -> Vec<bool> {
        vec![
            true,
            false, false, false, false, false,
            false, false, false, false, false,
            true,
        ]
    }

    fn length_type_id_1_2() -> Vec<bool> {
        vec![
            true,
            false, false, false, false, false,
            false, false, false, false, true,
            false,
        ]
    }

    fn length_type_id_1_3() -> Vec<bool> {
        vec![
            true,
            false, false, false, false, false,
            false, false, false, false, true,
            true,
        ]
    }

    fn literal_14() -> Vec<bool> {
        vec![false, true, true, true, false]
    }

    fn literal_15() -> Vec<bool> {
        vec![false, true, true, true, true]
    }

    fn packet_lit_14() -> Vec<bool> {
        let mut r = version2();
        r.extend(&type4());
        r.extend(&literal_14());
        r
    }

    fn packet_lit_15() -> Vec<bool> {
        let mut r = version2();
        r.extend(&type4());
        r.extend(&literal_15());
        r
    }

    fn filler2() -> Vec<bool> {
        vec![false, false]
    }

    #[test]
    fn literal_from() {
        {
            let raw_literal = vec![
                false, false, false, true,
            ];
            let literal: Literal = Literal::from(raw_literal);
            assert_eq!(literal.value(), 1);
        }
        {
            let raw_literal = vec![
                true, true, true, true,
            ];
            let literal: Literal = Literal::from(raw_literal);
            assert_eq!(literal.value(), 15);
        }
        {
            let raw_literal = vec![
                true, true, true, true,
                true, true, true, true,
            ];
            let literal: Literal = Literal::from(raw_literal);
            assert_eq!(literal.value(), 255);
        }
    }

    #[test]
    fn packet_parse_literals() {
        {
            let mut raw_bits: Vec<bool> = vec![
                false, false, false, false, true,
                false, false, false,
            ];
            let lit = Literal::parse(&mut raw_bits).unwrap();
            assert_eq!(raw_bits.len(), 3);
            assert_eq!(lit.raw.len(), 4);
        }
        {
            let mut raw_bits: Vec<bool> = vec![
                false, false, false, true, false,
                false, false, false, false, true,
                false, false, false, false, true,
                false, false, false,
            ];
            let lit = Literal::parse(&mut raw_bits).unwrap();
            assert_eq!(lit.raw.len(), 4);
            assert_eq!(raw_bits.len(), 13);
            Literal::parse(&mut raw_bits);
            assert_eq!(raw_bits.len(), 8);
            Literal::parse(&mut raw_bits);
            assert_eq!(raw_bits.len(), 3);
            let r = Literal::parse(&mut raw_bits);
            assert!(matches!(r, None));
        }
        {
            let mut raw_bits: Vec<bool> = vec![
                true, false, false, true, false,
                true, false, false, false, true,
                false, false, false, false, true,
                false, false, false,
            ];
            let lit = Literal::parse(&mut raw_bits).unwrap();
            assert_eq!(lit.raw.len(), 12);
            assert_eq!(raw_bits.len(), 3);
            let r = Literal::parse(&mut raw_bits);
            assert!(matches!(r, None));
        }
    }

    #[test]
    fn operator_parse() {
        {
            let mut raw_bits = vec![];
            raw_bits.extend(&length_type_id_0_11());
            raw_bits.extend(&packet_lit_15());
            raw_bits.extend(&filler2());
            println!("{:?}", raw_bits);
            let op = Operator::parse(&mut raw_bits).unwrap();
            assert_eq!(raw_bits.len(), 2);
            assert_eq!(op.children.len(), 1);
        }
        {
            let mut raw_bits: Vec<bool> = vec![
                true,
                false, false, false, false, false,
                false, false, false, false, false,
                true,
                false, false, false,
                true, false, false,
                false, true, true, true, true,
                false, false,
            ];
            Operator::parse(&mut raw_bits);
            assert_eq!(raw_bits.len(), 2);
        }
    }
    
    #[test]
    fn packet_parse() {
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&type4());
            raw.extend(&literal_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(raw.len(), 2);
            assert_eq!(packet.version, 2);
            assert_eq!(packet.type_id, 4);
            assert!(matches!(packet.child, Type::Literal(_)));
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&type5());
            raw.extend(&length_type_id_0_11());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            println!("{:?}", raw);
            assert_eq!(raw.len(), 2);
            assert_eq!(packet.version, 2);
            assert_eq!(packet.type_id, 5);
            assert!(matches!(packet.child, Type::Operator(_)));
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&type5());
            raw.extend(&length_type_id_1_1());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            println!("{:?}", raw);
            assert_eq!(raw.len(), 2);
            assert_eq!(packet.version, 2);
            assert_eq!(packet.type_id, 5);
            assert!(matches!(packet.child, Type::Operator(_)));
            if let Type::Operator(op) = packet.child {
                assert_eq!(op.children.len(), 1);
            }
        }
    }

    #[test]
    fn multiple_children() {
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&type5());
            raw.extend(&length_type_id_1_3());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(raw.len(), 2);
            assert_eq!(packet.version, 2);
            assert_eq!(packet.type_id, 5);
            assert!(matches!(packet.child, Type::Operator(_)));
            if let Type::Operator(op) = packet.child {
                assert_eq!(op.children.len(), 3);
            }
        }
    }

    #[test]
    fn add_versions() {
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&type5());
            raw.extend(&length_type_id_1_3());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.add_versions(), 8);
        }
    }

    #[test]
    fn operations() {
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(0));
            raw.extend(&length_type_id_1_3());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 45);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(1));
            raw.extend(&length_type_id_1_3());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 15*15*15);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(2));
            raw.extend(&length_type_id_1_3());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 15);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(3));
            raw.extend(&length_type_id_1_3());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 15);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(5));
            raw.extend(&length_type_id_1_2());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_14());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 1);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(5));
            raw.extend(&length_type_id_1_2());
            raw.extend(&packet_lit_14());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 0);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(6));
            raw.extend(&length_type_id_1_2());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_14());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 0);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(6));
            raw.extend(&length_type_id_1_2());
            raw.extend(&packet_lit_14());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 1);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(7));
            raw.extend(&length_type_id_1_2());
            raw.extend(&packet_lit_14());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 0);
        }
        {
            let mut raw = vec![];
            raw.extend(&version2());
            raw.extend(&typen(7));
            raw.extend(&length_type_id_1_2());
            raw.extend(&packet_lit_15());
            raw.extend(&packet_lit_15());
            raw.extend(&filler2());
            let packet = Packet::parse(&mut raw).unwrap();
            assert_eq!(packet.evaluate(), 1);
        }
    }
}
