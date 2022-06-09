use std::{io, io::Read,ops::{Add, Mul, Div, Rem}, collections::HashMap, fmt::Display};

const DEBUG: bool = false;

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Error reading stdin.");

    if DEBUG {
        display_alus(buf.clone());
    }

    let ins = InstructionList::<isize>::from(buf.clone()).split();
    let pcs: Vec<ParametrisedChunk> = ins.iter().map(|i| ParametrisedChunk::from(i.clone())).collect();
    let highest_route = top_route(pcs.iter().collect(), 0, &mut RouteCache(HashMap::new()), false).unwrap();
    let lowest_route = top_route(pcs.iter().collect(), 0, &mut RouteCache(HashMap::new()), true).unwrap();

    println!("Highest valid model number: {}\nLowest valid model number: {}",
        highest_route.0.iter().map(|x| x.n.to_string()).collect::<String>(),
        lowest_route.0.iter().map(|x| x.n.to_string()).collect::<String>(),
    );
}

#[derive(Eq, Hash, PartialEq)]
struct CacheKey(usize, isize);

#[derive(Clone)]
struct Route(Vec<Point>);

impl Route {
    fn add(&mut self, point: Point) {
        self.0.push(point);
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Point{z: isize, n: isize}

struct RouteCache(HashMap<CacheKey, Option<Route>>);

impl RouteCache {
    fn get(&self, key: &CacheKey) -> Option<&Option<Route>> {
        self.0.get(key)
    }

    fn insert(&mut self, k: CacheKey, v: Option<Route>) {
        self.0.insert(k, v); 
    }
}

fn top_route(
    pcs: Vec<&ParametrisedChunk>,
    target: isize,
    cache: &mut RouteCache,
    lowest: bool
) -> Option<Route> {
    let size = pcs.len();
    if let Some(r) = cache.get(&CacheKey(size, target)) {
        return r.clone();
    }
    if size == 0 {
        if target <= 100 {
            cache.insert(CacheKey(0,0), Some(Route(vec![])));
            return Some(Route(vec![]));
        } else {
            cache.insert(CacheKey(0,target), None);
            return None;
        }
    }
    let mut pcs2 = pcs.clone();
    let pc = pcs2.pop().unwrap();
    let res: Vec<Point> = pc.find_values(target);
    if res.is_empty() {
        cache.insert(CacheKey(size, target), None);
        return None;
    }
    let mut children: Vec<(Route, &Point)> = res.iter()
        .map(|x| (
            top_route(pcs2.clone(), x.z, cache, lowest),
            x
        )).filter(
            |(x,_)|
            x.is_some()
        ).map(|(x,y)| (x.unwrap(), y)).collect();
    if children.is_empty() {
        cache.insert(CacheKey(size, target), None);
        None
    } else {
        children.sort_by(|a, b| a.0.0[0].n.cmp(&b.0.0[0].n));
        if lowest {
            children.reverse();
        }
        let mut r: Route = children[0].0.clone();
        r.add(children[0].1.clone());
        cache.insert(CacheKey(size, target), Some(r.clone()));
        Some(r)
    }
}

fn display_alus(buf: String) -> Vec<Alu<Expr<Sym>>> {
    let ins = InstructionList::<Expr<Sym>>::from(buf);
    let mut v: Vec<Alu<Expr<Sym>>> = vec![];
    for list in ins.split() {
        let mut alu: Alu<Expr<Sym>> = Alu::new();
        alu.3 = Expr::Atom(Atom::Var(1));
        list.process_with_tidy(&mut alu, &mut 0);
        println!("{}", alu.3);
        v.push(alu);
    }
    v
}

#[derive(PartialEq, Clone)]
struct Register(usize);

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Register(0) => 'w',
            Register(1) => 'x',
            Register(2) => 'y',
            Register(3) => 'z',
            _ => unreachable!(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Sym(isize);

#[derive(Clone, PartialEq, Debug)]
enum Atom<T>{
    Value(T),
    Var(usize),
}

#[derive(Clone, PartialEq, Debug)]
enum Expr<T>{
    Atom(Atom<T>),
    BinOp(Op, Box<Expr<T>>, Box<Expr<T>>),
}

impl<T:Variable> Expr<T> {
    fn _bind(&self, bindings: &Vec<T>) -> Expr<T> {
        match self {
            Self::Atom(Atom::Var(n)) => {
                Self::Atom(Atom::Value(bindings[*n].clone()))
            },
            Self::BinOp(op, e1, e2) => {
                let (s1, s2) = (e1._bind(bindings), e2._bind(bindings));
                Self::BinOp(op.clone(), Box::new(s1), Box::new(s2))
            },
            _ => {
                self.clone()
            }
        }
    }

    fn _evaluate(&self) -> T {
        match self {
            Self::Atom(Atom::Var(_)) => panic!(),
            Self::Atom(Atom::Value(v)) => v.clone(),
            Self::BinOp(op, e1, e2) => {
                let (s1, s2) = (e1._evaluate(), e2._evaluate());
                T::map(op.clone(), s1, s2)
            }
        }
    }

    fn simplify(&self) -> Self {
        let mut ret = self.clone();
        loop {
            let simplified = match self {
                Self::Atom(_) => {
                    self.clone()
                },
                Self::BinOp(op, e1, e2) => {
                    let (s1, s2) = (e1.simplify(), e2.simplify());
                    let default = Self::BinOp(op.clone(), Box::new(s1.clone()), Box::new(s2.clone()));
                    match (op, &s1, &s2) {
                        (Op::Add, Expr::Atom(Atom::Value(x)), _) => {
                            if *x == T::zero() {
                                s2
                            } else {
                                default
                            }
                        },
                        (Op::Add, _, Expr::Atom(Atom::Value(x))) => {
                            if *x == T::zero() {
                                s1
                            } else {
                                default
                            }
                        },
                        (Op::Mul, Expr::Atom(Atom::Value(x1)), Expr::Atom(Atom::Value(x2))) => {
                            if *x1 == T::one() {
                                s2
                            } else if *x1 == T::zero() || *x2 == T::zero() {
                                Self::Atom(Atom::Value(T::zero()))
                            } else if *x2 == T::one() {
                                s1
                            } else {
                                default
                            }
                        },
                        (Op::Mul, Expr::Atom(Atom::Value(x)), _) => {
                            if *x == T::one() {
                                s2
                            } else if *x == T::zero() {
                                Self::Atom(Atom::Value(T::zero()))
                            } else {
                                default
                            }
                        },
                        (Op::Mul, _, Expr::Atom(Atom::Value(x))) => {
                            if *x == T::one() {
                                s1
                            } else if *x == T::zero() {
                                Self::Atom(Atom::Value(T::zero()))
                            } else {
                                default
                            }
                        },
                        (Op::Div, _, Expr::Atom(Atom::Value(x))) => {
                            if *x == T::one() {
                                s1
                            } else {
                                default
                            }
                        },
                        (Op::Div, Expr::Atom(Atom::Value(x)), _) => {
                            if *x == T::zero() {
                                Expr::Atom(Atom::Value(T::zero()))
                            } else {
                                default
                            }
                        },
                        (Op::Mod, Expr::Atom(Atom::Value(x)), _) => {
                            if *x == T::zero() {
                                Expr::Atom(Atom::Value(T::zero()))
                            } else {
                                default
                            }
                        },
                        (Op::Eql, Expr::Atom(Atom::Var(x1)), Expr::Atom(Atom::Var(x2))) => {
                            if x1 == x2 {
                                Expr::Atom(Atom::Value(T::one()))
                            } else {
                                default
                            }
                        },
                        (Op::Eql, Expr::Atom(Atom::Value(x1)), Expr::Atom(Atom::Value(x2))) => {
                            if x1 != x2 {
                                Expr::Atom(Atom::Value(T::zero()))
                            } else {
                                Expr::Atom(Atom::Value(T::one()))
                            }
                        },
                        (_, _, _) => default,
                    }
                },
            };
            if ret == simplified {
                break;
            }
            ret = simplified;
        }
        ret
    }
}

#[derive(PartialEq, Clone, Debug)]
enum Op{
    Add,
    Mul,
    Div,
    Mod,
    Eql,
}

impl Display for Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Sym(x) => x.to_string(),
        };
        write!(f, "{}", r)
    }
}

impl<T:Variable> Display for Atom<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(v) => write!(f, "{}", v),
            Self::Var(n) => write!(f, "n{}", n),
        }
    }
}

impl<T:Variable> Display for Expr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Atom(a) => {
                a.fmt(f)
            },
            Expr::BinOp(op, e1, e2) => {
                write!(f, "{}({},{})", op, e1, e2)
            }
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Op::Add => "+",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Mod => "%",
            Op::Eql => "==",
        })
    }
}

impl Add for Sym {
    type Output = Expr<Sym>;
    fn add(self, rhs: Self) -> Self::Output {
        Expr::add(Expr::Atom(Atom::Value(self)), Expr::Atom(Atom::Value(rhs)))
    }
}

impl Mul for Sym {
    type Output = Expr<Sym>;
    fn mul(self, rhs: Self) -> Self::Output {
        Expr::mul(Expr::Atom(Atom::Value(self)), Expr::Atom(Atom::Value(rhs)))
    }
}

impl Div for Sym {
    type Output = Expr<Sym>;
    fn div(self, rhs: Self) -> Self::Output {
        Expr::div(Expr::Atom(Atom::Value(self)), Expr::Atom(Atom::Value(rhs)))
    }
}

impl Rem for Sym {
    type Output = Expr<Sym>;
    fn rem(self, rhs: Self) -> Self::Output {
        Expr::rem(Expr::Atom(Atom::Value(self)), Expr::Atom(Atom::Value(rhs)))
    }
}

impl From<isize> for Sym {
    fn from(a: isize) -> Self {
        Sym(a)
    }
}

impl<T> Add for Expr<T> {
    type Output = Expr<T>;
    fn add(self, rhs: Self) -> Self::Output {
        Expr::BinOp(Op::Add, Box::new(self), Box::new(rhs))
    }
}

impl<T> Mul for Expr<T> {
    type Output = Expr<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        Expr::BinOp(Op::Mul, Box::new(self), Box::new(rhs))
    }
}

impl<T> Div for Expr<T> {
    type Output = Expr<T>;
    fn div(self, rhs: Self) -> Self::Output {
        Expr::BinOp(Op::Div, Box::new(self), Box::new(rhs))
    }
}

impl<T:Variable> Rem for Expr<T> {
    type Output = Expr<T>;
    fn rem(self, rhs: Self) -> Self::Output {
        Expr::BinOp(Op::Mod, Box::new(self), Box::new(rhs))
    }
}

impl Variable for Sym {
    fn zero() -> Self {
        Sym(0)
    }
    fn one() -> Self {
        Sym(1)
    }
    fn map(op: Op, arg1: Self, arg2: Self) -> Self {
        let (Sym(a1), Sym(a2)) = (arg1, arg2);
        Sym(isize::map(op, a1, a2))
    }
}

trait Variable: PartialEq + Display + Clone + Add + Mul + Div + Rem + From<isize> + core::fmt::Debug {
    fn zero() -> Self;
    fn one() -> Self;
    fn map(op: Op, arg1: Self, arg2: Self) -> Self;
}

impl Variable for isize {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
    fn map(op: Op, arg1: Self, arg2: Self) -> Self {
        match op {
            Op::Add => arg1 + arg2,
            Op::Mul => arg1 * arg2,
            Op::Div => arg1 / arg2,
            Op::Mod => arg1 % arg2,
            Op::Eql => if arg1 == arg2 {Self::one()} else {Self::zero()},
        }
    }
}

impl<T:Variable> From<isize> for Expr<T> {
    fn from(a: isize) -> Self {
        Expr::Atom(Atom::Value(a.into()))
    }
}

impl<T:Variable> Variable for Expr<T> {
    fn one() -> Self {
        Expr::Atom(Atom::Value(T::one()))
    }
    fn zero() -> Self {
        Expr::Atom(Atom::Value(T::zero()))
    }
    fn map(op: Op, arg1: Self, arg2: Self) -> Self {
        Expr::BinOp(op, Box::new(arg1), Box::new(arg2))
    }
}

#[derive(PartialEq, Clone, Debug)]
struct Alu<T:Variable>(T, T, T, T);

impl<T:Variable> From<[T; 4]> for Alu<T> {
    fn from(a: [T; 4]) -> Self {
        Self(a[0].clone(), a[1].clone(), a[2].clone(), a[3].clone())
    }
}

impl<T:Variable> Alu<T> {
    fn new() -> Self {
        Self(T::zero(), T::zero(), T::zero(), T::zero())
    }

    fn realise(&self, v: &Value<T>) -> T {
        match v {
            Value::Atom(v) => v.clone(),
            Value::Register(r) => self.get(r),
        }
    }

    fn get(&self, r: &Register) -> T {
        match r.0 {
            0 => self.0.clone(),
            1 => self.1.clone(),
            2 => self.2.clone(),
            3 => self.3.clone(),
            _ => unreachable!(),
        }
    }

    fn store(&mut self, r: &Register, value: T) {
        match r.0 {
            0 => { self.0 = value; },
            1 => { self.1 = value; },
            2 => { self.2 = value; },
            3 => { self.3 = value; },
            _ => unreachable!(),
        }
    }
}

impl<T:Variable> Alu<Expr<T>> {
    fn tidy(&mut self) {
        self.0 = self.0.simplify();
        self.1 = self.1.simplify();
        self.2 = self.2.simplify();
        self.3 = self.3.simplify();
    }
}

impl<T:Variable> Display for Alu<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{},{},{}]", self.0, self.1, self.2, self.3)
    }
}

#[derive(PartialEq, Clone)]
enum Value<T> {
    Register(Register),
    Atom(T),
}

impl<T:Variable> Display for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Register(r) => write!(f, "{}", r),
            Value::Atom(a) => write!(f, "{}", a),
        }
    }
}
#[derive(PartialEq, Clone)]
enum Instruction<T> {
    Inp(Register),
    Add(Register, Value<T>),
    Mul(Register, Value<T>),
    Div(Register, Value<T>),
    Mod(Register, Value<T>),
    Eql(Register, Value<T>),
}

impl<T:Variable> Instruction<Expr<T>>
    {
    fn process_instruction(&self, alu: &mut Alu<Expr<T>>, inputs: &mut usize) {
        match self {
            Instruction::Inp(r) => {
                let value = Expr::Atom(Atom::Var(*inputs));
                *inputs += 1;
                alu.store(r, value);
            },
            Instruction::Add(r, v) => {
                alu.store(r, alu.get(r) + alu.realise(v));
            },
            Instruction::Mul(r, v) => {
                alu.store(r, alu.get(r) * alu.realise(v));
            },
            Instruction::Div(r, v) => {
                alu.store(r, alu.get(r) / alu.realise(v));
            },
            Instruction::Mod(r, v) => {
                alu.store(r, alu.get(r) % alu.realise(v));
            },
            Instruction::Eql(r, v) => {
                alu.store(r, Expr::BinOp(Op::Eql, Box::new(alu.get(r)), Box::new(alu.realise(v))));
            },
        }
    }

}

impl<T:Variable> Display for Instruction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Inp(r) => write!(f, "inp {}", r),
            Instruction::Add(r, v) => write!(f, "add {} {}", r, v),
            Instruction::Mul(r, v) => write!(f, "mul {} {}", r, v),
            Instruction::Div(r, v) => write!(f, "div {} {}", r, v),
            Instruction::Mod(r, v) => write!(f, "mod {} {}", r, v),
            Instruction::Eql(r, v) => write!(f, "eql {} {}", r, v),
        }
    }
}

impl<T:Variable> From<&str> for Instruction<T> {
    fn from(s: &str) -> Self {
        let v: Vec<&str> = s.split(' ').collect();
        let register = match v[1] {
            "w" => Register(0),
            "x" => Register(1),
            "y" => Register(2),
            "z" => Register(3),
            _ => unreachable!(),
        };
        let value = if v.len() == 3 {
            match v[2] {
                "w" => Value::Register(Register(0)),
                "x" => Value::Register(Register(1)),
                "y" => Value::Register(Register(2)),
                "z" => Value::Register(Register(3)),
                _ => Value::Atom(T::from(v[2].parse::<isize>().unwrap())),
            }
        } else { Value::Atom(0_isize.into()) };
        match v[0] {
            "inp" => Self::Inp(register),
            "add" => Self::Add(register, value),
            "mul" => Self::Mul(register, value),
            "div" => Self::Div(register, value),
            "mod" => Self::Mod(register, value),
            "eql" => Self::Eql(register, value),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
struct InstructionList<T>(Vec<Instruction<T>>);

impl<T:Variable> Display for InstructionList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    where InstructionList<T>:Display {
        write!(f, "{}", self.0.iter().map(|i| format!("{} ", *i)).collect::<String>())
    }
}
impl<T:Variable> InstructionList<Expr<T>> {
    fn process(&self, alu: &mut Alu<Expr<T>>, inputs: &mut usize) {
        for i in &self.0 {
            i.process_instruction(alu, inputs);
        }
    }

    fn process_with_tidy(&self, alu: &mut Alu<Expr<T>>, inputs: &mut usize) {
        self.process(alu, inputs);
        alu.tidy();
    }
}

impl<T> InstructionList<T> {
    fn split(&self) -> Vec<InstructionList<T>> where T: Clone {
        let iter = self.0.iter();
        let mut r = vec![];
        let mut v: Vec<Instruction<T>> = vec![];
        for i in iter {
            if let Instruction::Inp(_) = i {
                if !v.is_empty() {
                    r.push(InstructionList(v));
                }
                v = vec![i.clone()];
            } else {
                v.push(i.clone());
            }
        }
        r.push(InstructionList(v));
        r
    }
}

impl<T:Variable> From<String> for InstructionList<T> {
    fn from(s: String) -> Self {
        let instructions = s.trim().split('\n')
        .map(Instruction::from).collect();
        Self(instructions)
    }
}

#[derive(Clone)]
struct ParametrisedChunk{
    divide: bool,
    offset: isize,
    addition: isize,
}

impl Display for ParametrisedChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", if self.divide {'/'} else {'*'},self.offset, self.addition)
    }
}

impl ParametrisedChunk {
    fn process(&self, z: isize, n: isize) -> isize {
        let y = z / self.divisor();
        if (z % 26) + self.offset == n {
            y
        } else {
            26 * y + n + self.addition
        }
    }

    fn divisor(&self) -> isize {
        if self.divide { 26 } else { 1 }
    }

    /*
        multi-inverse of self.process()
        case1: divide, special
            z2 = z/26, z = 26*z2+0..26, n=(z%26)+offset
        case2: divide, normal
            z2 = 26*(z/26)+n+a !=> (z2-n-a)%26 = 0 => z2-a-n=26*m => m=z/26, z=26*m+0..26=z2-a-n+0..26  z = (z2-n-a)/26*26+0..26 , (z%26)+offset<>n

        case3: non, special
            z2 = z, z = z2, n=(z%26)+offset = (z2%26)+offset
        case4: non, normal
            z2 = 26*z+n+a => z = (z2-n-a)/26, (z%26)+offset<>n
    */
    fn find_values(&self, target: isize) -> Vec<Point> {
        let mut r = vec![];
        if self.divide {
            let mut temp = vec![];
            for i in 0..26 {
                let z = 26*target+i;
                let n = (z%26)+self.offset;
                if (1..=9).contains(&n) && z>=0 && self.process(z, n) == target {
                    temp.push(Point{z,n});
                }
            }
            r.append(&mut temp);
            for i in 0..26 {
                for n in 1..=9 {
                    let z = target - n - self.addition + i;
                    if z >= 0 && self.process(z, n) == target {
                        temp.push(Point{z,n});
                    }
                }
            }
            r.append(&mut temp);
        } else {
            let mut temp = vec![];
            {
                let z = target;
                let n = (target%26) + self.offset;
                if (1..=9).contains(&n) && z>=0 && self.process(z, n) == target {
                    temp.push(Point{z,n});
                }
            }
            {
                for n in 1..=9 {
                    let z = (target-n-self.addition)/26;
                    if z >= 0 && self.process(z, n) == target {
                        temp.push(Point{z,n});
                    }
                }
            }
            r.append(&mut temp);
        }
        r.sort_unstable();
        r.dedup();
        r
    }
}

impl From<InstructionList<isize>> for ParametrisedChunk {
    fn from(ins: InstructionList<isize>) -> Self {
        let divide = match ins.0[4] {
            Instruction::<isize>::Div(_, Value::Atom(x)) => {
                x != 1
            },
            _ => unreachable!(),
        };
        let offset = match ins.0[5] {
            Instruction::<isize>::Add(_, Value::Atom(x)) => {
                x
            },
            _ => unreachable!(),
        };
        let addition = match ins.0[15] {
            Instruction::<isize>::Add(_, Value::Atom(x)) => {
                x
            },
            _ => unreachable!(),
        };
        Self { divide, offset, addition }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exprs() {
        {
            let expr: Expr<Sym> = Expr::BinOp(Op::Eql, Box::new(Expr::Atom(Atom::Var(1))), Box::new(Expr::Atom(Atom::Var(2))));
            assert_eq!(expr.to_string(), "==(n1,n2)");
        }
        {
            let expr: Expr<Sym> = Expr::BinOp(Op::Add, Box::new(Expr::Atom(Atom::Var(1))), Box::new(Expr::Atom(Atom::Var(2))));
            assert_eq!(expr.to_string(), "+(n1,n2)");
        }
        {
            let expr: Expr<Sym> = Expr::BinOp(Op::Add, Box::new(Expr::Atom(Atom::Var(1))), Box::new(Expr::BinOp(Op::Div, Box::new(Expr::Atom(Atom::Var(1))), Box::new(Expr::Atom(Atom::Var(2))))));
            assert_eq!(expr.to_string(), "+(n1,/(n1,n2))");
        }
    }

    #[test]
    fn read_instruction() {
        {
            let s = "mul x 0";
            let i: Instruction<Expr<Sym>> = s.into();
            assert_eq!(i.to_string(), s);
        }
        {
            let s = "inp x";
            let i: Instruction<Expr<Sym>> = s.into();
            assert_eq!(i.to_string(), s);
        }
    }

    #[test]
    fn symbolic_alu() {
        let expr: Expr<Sym> = Expr::BinOp(Op::Add, Box::new(Expr::Atom(Atom::Var(1))), Box::new(Expr::BinOp(Op::Div, Box::new(Expr::Atom(Atom::Var(1))), Box::new(Expr::Atom(Atom::Var(2))))));
        let mut alu: Alu<Expr<Sym>> = Alu::new();
        alu.0 = expr;
        assert_eq!(alu.to_string(), "[+(n1,/(n1,n2)),0,0,0]");
        let s = "mul w 2";
        let i: Instruction<Expr<Sym>> = s.into();
        i.process_instruction(&mut alu, &mut 0);
        assert_eq!(alu.to_string(), "[*(+(n1,/(n1,n2)),2),0,0,0]");
    }

    #[test]
    fn symbolic_list_processing() {
        let s = "mul w 2\n\
            add x 3\n\
            add y 3\n\
            mul x 3\n\
            mod x 3\n\
            div x y\n\
            add y 4\n".to_string();
        let ins: InstructionList<Expr<Sym>> = s.into();
        let mut alu: Alu<Expr<Sym>> = Alu::new();
        ins.process(&mut alu, &mut 0);
        assert_eq!(alu.to_string(), "[*(0,2),/(%(*(+(0,3),3),3),+(0,3)),+(+(0,3),4),0]");
    }

    #[test]
    fn symbolic_list_tidy_processing() {
        let s = "mul w 2\n\
            add x 3\n\
            add y 3\n\
            mul x 3\n\
            mod x 3\n\
            div x y\n\
            add y 4\n".to_string();
        let ins: InstructionList<Expr<Sym>> = s.into();
        let mut alu: Alu<Expr<Sym>> = Alu::new();
        ins.process_with_tidy(&mut alu, &mut 0);
        assert_eq!(alu.to_string(), "[0,/(%(*(3,3),3),3),+(3,4),0]");
    }

    #[test]
    fn handle_inp() {
        let s = "mul w 2\n\
            inp z\n\
            inp x\n\
            add y 3\n\
            mul x 3\n\
            add y 4\n".to_string();
        let ins: InstructionList<Expr<Sym>> = s.into();
        let mut alu: Alu<Expr<Sym>> = Alu::new();
        ins.process(&mut alu, &mut 0);
        assert_eq!(alu.to_string(), "[*(0,2),*(n1,3),+(+(0,3),4),n0]");
    }

    #[test]
    fn simplify() {
        {
            let expr: Expr<Sym> =
                Expr::BinOp(
                    Op::Mul,
                    Box::new(Expr::Atom(Atom::Value(Sym::zero()))),
                    Box::new(Expr::Atom(Atom::Var(2))),
                );
            let res = expr.simplify();
            assert_eq!(res, Expr::Atom(Atom::Value(Sym::zero())));
        }
        {
            let expr: Expr<Sym> =
                Expr::BinOp(
                    Op::Mul,
                    Box::new(Expr::Atom(Atom::Var(2))),
                    Box::new(Expr::Atom(Atom::Value(Sym::zero()))),
                );
            let res = expr.simplify();
            assert_eq!(res, Expr::Atom(Atom::Value(Sym::zero())));
        }
        {
            let expr: Expr<Sym> =
                Expr::BinOp(
                    Op::Add,
                    Box::new(Expr::Atom(Atom::Value(Sym::zero()))),
                    Box::new(Expr::Atom(Atom::Var(2))),
                );
            let res = expr.simplify();
            assert_eq!(res, Expr::Atom(Atom::Var(2)));
        }
        {
            let expr: Expr<Sym> =
                Expr::BinOp(
                    Op::Mul,
                    Box::new(Expr::Atom(Atom::Value(Sym::one()))),
                    Box::new(Expr::Atom(Atom::Var(2))),
                );
            let res = expr.simplify();
            assert_eq!(res, Expr::Atom(Atom::Var(2)));
        }
        {
            let expr: Expr<Sym> =
                Expr::BinOp(
                    Op::Div,
                    Box::new(Expr::Atom(Atom::Var(2))),
                    Box::new(Expr::Atom(Atom::Value(Sym::one()))),
                );
            let res = expr.simplify();
            assert_eq!(res, Expr::Atom(Atom::Var(2)));
        }
        {
            let expr: Expr<Sym> =
                Expr::BinOp(
                    Op::Div,
                    Box::new(Expr::Atom(Atom::Value(Sym::zero()))),
                    Box::new(Expr::Atom(Atom::Var(2))),
                );
            let res = expr.simplify();
            assert_eq!(res, Expr::Atom(Atom::Value(Sym::zero())));
        }
        {
            let expr: Expr<Sym> =
                Expr::BinOp(
                    Op::Eql,
                    Box::new(Expr::Atom(Atom::Var(2))),
                    Box::new(Expr::Atom(Atom::Var(2))),
                );
            let res = expr.simplify();
            assert_eq!(res, Expr::Atom(Atom::Value(Sym::one())));
        }
        {
            let expr: Expr<Sym> =
                Expr::BinOp(
                    Op::Eql,
                    Box::new(Expr::Atom(Atom::Value(Sym(2)))),
                    Box::new(Expr::Atom(Atom::Value(Sym(3)))),
                );
            let res = expr.simplify();
            assert_eq!(res, Expr::Atom(Atom::Value(Sym::zero())));
        }
    }

    #[test]
    fn simplify_nested() {
        let expr: Expr<Sym> =
            Expr::BinOp(
                Op::Add,
                Box::new(Expr::BinOp(
                    Op::Mul, Box::new(Expr::Atom(Atom::Value(Sym(25)))), Box::new(Expr::Atom(Atom::Value(Sym(0))))
                )),
                Box::new(Expr::Atom(Atom::Value(Sym(1)))),
            );
        let res = expr.simplify();
        assert_eq!(res, Expr::Atom(Atom::Value(Sym::one())));
    }

    #[test]
    fn mods() {
        let expr: Expr<Sym> =
            Expr::BinOp(
                Op::Mod,
                Box::new(Expr::Atom(Atom::Value(Sym(25)))),
                Box::new(Expr::Atom(Atom::Value(Sym(4)))),
            );
        let res = expr.simplify();
        let mut alu: Alu<Expr<Sym>> = Alu::new();
        alu.0 = res;
        assert_eq!(alu.0.to_string(), "%(25,4)");
        let s = "add w 7\n\
            add z 3\n\
            mod w z".to_string();
        let ins: InstructionList<Expr<Sym>> = s.into();
        alu = Alu::new();
        ins.process(&mut alu, &mut 0);
        assert_eq!(alu.to_string(), "[%(+(0,7),+(0,3)),0,0,+(0,3)]");
    }

    #[test]
    fn bind() {
        let expr = Expr::BinOp(
            Op::Add,
            Box::new(Expr::Atom(Atom::Var(0))),
            Box::new(Expr::Atom(Atom::Value(Sym(4)))),
        );
        let bindings: Vec<Sym> = vec![Sym(3)];
        let res = expr._bind(&bindings);
        let target = Expr::BinOp(
            Op::Add,
            Box::new(Expr::Atom(Atom::Value(Sym(3)))),
            Box::new(Expr::Atom(Atom::Value(Sym(4)))),
        );
        assert_eq!(res, target);
        assert_eq!(res._evaluate().0, 7);
    }

    #[test]
    fn parametrised_chunks() {
        let s = "inp w\n\
            mul x 0\n\
            add x z\n\
            mod x 26\n\
            div z 1\n\
            add x 15\n\
            eql x w\n\
            eql x 0\n\
            mul y 0\n\
            add y 25\n\
            mul y x\n\
            add y 1\n\
            mul z y\n\
            mul y 0\n\
            add y w\n\
            add y 9\n\
            mul y x\n\
            add z y";
        let ins = InstructionList::from(s.to_string());
        let chunk: ParametrisedChunk = ParametrisedChunk::from(ins);
        assert!(!chunk.divide);
        assert_eq!(chunk.offset, 15);
        assert_eq!(chunk.addition, 9);
        let ins2: InstructionList<Expr<isize>> = InstructionList::from(s.to_string());

        {
            let res = chunk.process(0, 0);
            assert_eq!(res, 9);
            let mut alu = Alu::new();
            ins2.process_with_tidy(&mut alu, &mut 0);
            let bindings = vec![0,0];
            assert_eq!(alu.3._bind(&bindings)._evaluate(), 9);
        }
        {
            let res = chunk.process(27, 1);
            assert_eq!(res, 27*26 + 1 + 9);
            let mut alu: Alu<Expr<isize>> = Alu::new();
            alu.3 = Expr::Atom(Atom::Var(1));
            ins2.process_with_tidy(&mut alu, &mut 0);
            let bindings = vec![1,27];
            assert_eq!(alu.3._bind(&bindings)._evaluate(), 712);
        }
    }

    fn instruction(d: bool, o: isize, a: isize) -> String {
        format!("inp w\n\
            mul x 0\n\
            add x z\n\
            mod x 26\n\
            div z {}\n\
            add x {}\n\
            eql x w\n\
            eql x 0\n\
            mul y 0\n\
            add y 25\n\
            mul y x\n\
            add y 1\n\
            mul z y\n\
            mul y 0\n\
            add y w\n\
            add y {}\n\
            mul y x\n\
            add z y", if d {26} else {1}, o, a)
    }

    #[test]
    fn mutate_pcs() {
        for s in [
            (false, 15, 9),
            (true, 15, 9),
            (false, -15, 9),
            (true, -15, 9),
        ] {
            let ins = InstructionList::from(instruction(s.0, s.1, s.2));
            let chunk: ParametrisedChunk = ParametrisedChunk::from(ins);
            assert_eq!(chunk.divide, s.0);
            assert_eq!(chunk.offset, s.1);
            assert_eq!(chunk.addition, s.2);
            for (z, n) in [
                (0,1),(0,2),(0,3),(0,4),(0,5),(0,6),(0,7),(0,8),(0,9),
                (10,5),(20,5),(30,5),(40,5),(50,5),
            ] {
                let ins2: InstructionList<Expr<isize>> = InstructionList::from(instruction(s.0, s.1, s.2));
                println!("({},{},{}): {},{}",s.0,s.1,s.2,z,n);
                let res = chunk.process(z, n);
                let mut alu = Alu::new();
                alu.3 = Expr::Atom(Atom::Value(z));
                ins2.process_with_tidy(&mut alu, &mut 0);
                let bindings = vec![n];
                println!("{}", alu.3._bind(&bindings));
                let res2 = alu.3._bind(&bindings)._evaluate();
                assert_eq!(res2, res);
            }
        }

    }

    #[test]
    fn find_values() {
        {
            let target = 0;
            let chunk = ParametrisedChunk{
                divide: true,
                offset: -2,
                addition: 9,
            };
            let res = chunk.find_values(target);
            let expected: Vec<Point> = (3..=11).zip(1..=9).map(|(z,n)| Point{z,n}).collect();
            assert_eq!(res, expected);
        }
        {
            let target = 36;
            let chunk = ParametrisedChunk{
                divide: false,
                offset: -2,
                addition: 9,
            };
            let res = chunk.find_values(target);
            assert_eq!(res, vec![Point{z:1,n:1},Point{z:36,n:8}]);
        }
    }

    #[test]
    #[ignore] // longer-running
    fn all_inverses() {
        for divide in vec![true, false] {
            for offset in -25..25 {
                for addition in 0..16 {
                    let pc = ParametrisedChunk{divide, offset, addition};
                    for z in vec![0, 10, 20] {
                        for n in 1..=9 {
                            let res = pc.process(z, n);
                            let res2 = pc.find_values(res);
                            println!("{} ({},{})=={}",pc,z,n,res);
                            println!("{:?}", res2);
                            assert!(res2.contains(&Point{z, n}));
                        }
                    }
                    for target in vec![0, 10, 20] {
                        for p in pc.find_values(target) {
                            println!("{} ({},{})=={}",pc,p.z,p.n,target);
                            let res = pc.process(p.z, p.n);
                            println!("{}", res);
                            assert_eq!(res, target);
                        }
                    }
                }
            }
        }
    }

}