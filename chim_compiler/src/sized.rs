// ==================== 固定大小数组和类型级整数 ====================
// 参考 Agda 的依赖类型，支持类型中的大小信息

pub mod sized {
    use crate::stdlib::prelude::{Option, Result, Vec, HashMap, String, StringBuilder};

    // ==================== 类型级整数 ====================

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TypeNat(pub i128);

    impl TypeNat {
        pub fn zero() -> Self {
            TypeNat(0)
        }

        pub fn one() -> Self {
            TypeNat(1)
        }

        pub fn from_usize(n: usize) -> Self {
            TypeNat(n as i128)
        }

        pub fn to_usize(&self) -> Option<usize> {
            if self.0 >= 0 {
                Some(self.0 as usize)
            } else {
                None
            }
        }

        pub fn add(&self, other: &TypeNat) -> TypeNat {
            TypeNat(self.0 + other.0)
        }

        pub fn sub(&self, other: &TypeNat) -> Option<TypeNat> {
            if self.0 >= other.0 {
                Some(TypeNat(self.0 - other.0))
            } else {
                None
            }
        }

        pub fn mul(&self, other: &TypeNat) -> TypeNat {
            TypeNat(self.0 * other.0)
        }

        pub fn min(&self, other: &TypeNat) -> TypeNat {
            TypeNat(std::cmp::min(self.0, other.0))
        }

        pub fn max(&self, other: &TypeNat) -> TypeNat {
            TypeNat(std::cmp::max(self.0, other.0))
        }

        pub fn is_zero(&self) -> bool {
            self.0 == 0
        }

        pub fn is_positive(&self) -> bool {
            self.0 > 0
        }
    }

    // ==================== 类型级自然数 ====================

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Nat {
        Z,                    // Zero
        S(Box<Nat>),          // Successor
        Val(TypeNat),         // Concrete value
    }

    impl Nat {
        pub fn zero() -> Self {
            Nat::Z
        }

        pub fn one() -> Self {
            Nat::S(Box::new(Nat::Z))
        }

        pub fn from_usize(n: usize) -> Self {
            let mut result = Nat::Z;
            for _ in 0..n {
                result = Nat::S(Box::new(result));
            }
            result
        }

        pub fn to_usize(&self) -> Option<usize> {
            let mut count = 0;
            let mut current = self;
            loop {
                match current {
                    Nat::Z => return Some(count),
                    Nat::S(n) => {
                        count += 1;
                        current = n.as_ref();
                    }
                    Nat::Val(tn) => return tn.to_usize(),
                }
            }
        }

        pub fn add(&self, other: &Nat) -> Nat {
            let mut result = other.clone();
            let mut current = self;
            loop {
                match current {
                    Nat::Z => break,
                    Nat::S(n) => {
                        result = Nat::S(Box::new(result));
                        current = n.as_ref();
                    }
                    Nat::Val(tn) => {
                        if let Some(val) = tn.to_usize() {
                            for _ in 0..val {
                                result = Nat::S(Box::new(result));
                            }
                        }
                        break;
                    }
                }
            }
            result
        }

        pub fn sub(&self, other: &Nat) -> Option<Nat> {
            let self_val = self.to_usize()?;
            let other_val = other.to_usize()?;
            if self_val >= other_val {
                Some(Nat::from_usize(self_val - other_val))
            } else {
                None
            }
        }

        pub fn mul(&self, other: &Nat) -> Nat {
            let self_val = self.to_usize().unwrap_or(0);
            let other_val = other.to_usize().unwrap_or(0);
            Nat::from_usize(self_val * other_val)
        }
    }

    // ==================== 固定大小数组 ====================

    #[derive(Debug, Clone, PartialEq)]
    pub struct SizedVec<T, N: Nat> {
        pub data: Vec<T>,
        pub phantom: std::marker::PhantomData<N>,
    }

    impl<T, N> SizedVec<T, N>
    where
        N: Clone,
    {
        pub fn new() -> Self
        where
            N: Default,
        {
            SizedVec {
                data: Vec::new(),
                phantom: std::marker::PhantomData,
            }
        }

        pub fn with_capacity(capacity: N) -> Self {
            let cap = capacity.to_usize().unwrap_or(0);
            SizedVec {
                data: Vec::with_capacity(cap),
                phantom: std::marker::PhantomData,
            }
        }

        pub fn from_slice(slice: &[T]) -> Option<SizedVec<T, N>>
        where
            N: Nat + Default,
        {
            let expected_len = N::default().to_usize()?;
            if slice.len() == expected_len {
                Some(SizedVec {
                    data: slice.to_vec(),
                    phantom: std::marker::PhantomData,
                })
            } else {
                None
            }
        }

        pub fn from_vec(data: Vec<T>) -> Option<SizedVec<T, N>>
        where
            N: Nat + Default,
        {
            let expected_len = N::default().to_usize()?;
            if data.len() == expected_len {
                Some(SizedVec {
                    data,
                    phantom: std::marker::PhantomData,
                })
            } else {
                None
            }
        }

        pub fn len(&self) -> N
        where
            N: Default + Clone,
        {
            N::default()
        }

        pub fn is_empty(&self) -> bool
        where
            N: Nat,
        {
            self.data.is_empty()
        }

        pub fn as_slice(&self) -> &[T] {
            &self.data
        }

        pub fn as_mut_slice(&mut self) -> &mut [T] {
            &mut self.data
        }

        pub fn get(&self, index: usize) -> Option<&T> {
            self.data.get(index)
        }

        pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
            self.data.get_mut(index)
        }

        pub fn index(&self, index: N) -> Option<&T>
        where
            N: Nat + PartialEq<usize>,
        {
            let idx = index.to_usize()?;
            self.data.get(idx)
        }
    }

    impl<T, N> std::ops::Index<usize> for SizedVec<T, N> {
        type Output = T;
        
        fn index(&self, index: usize) -> &T {
            &self.data[index]
        }
    }

    impl<T, N> std::ops::IndexMut<usize> for SizedVec<T, N> {
        fn index_mut(&mut self, index: usize) -> &mut T {
            &mut self.data[index]
        }
    }

    // ==================== 类型级证明 ====================

    #[derive(Debug, Clone)]
    pub struct Proof<T> {
        pub value: T,
        pub constraints: Vec<Constraint>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Constraint {
        NatEq(Nat, Nat),
        NatLe(Nat, Nat),
        NatLt(Nat, Nat),
        NatGe(Nat, Nat),
        NatGt(Nat, Nat),
    }

    impl Proof<()> {
        pub fn trivial() -> Proof<()> {
            Proof {
                value: (),
                constraints: Vec::new(),
            }
        }

        pub fn assert(cond: bool) -> Option<Proof<()>> {
            if cond {
                Some(Proof::trivial())
            } else {
                None
            }
        }
    }

    // ==================== 依赖类型函数 ====================

    pub mod dep_fn {
        use super::{Nat, TypeNat, Proof};

        /// 依赖于自然数的函数类型
        pub struct DepFn<N: Nat, T> {
            pub phantom: std::marker::PhantomData<N>,
            pub output: T,
        }

        impl<N: Nat, T> DepFn<N, T> {
            pub fn new() -> Self {
                DepFn {
                    phantom: std::marker::PhantomData,
                    output: std::marker::PhantomData::default(),
                }
            }
        }

        /// 恒等函数（类型依赖于 n）
        pub fn id<T, N: Nat>(x: T) -> T {
            x
        }

        /// 常数函数
        pub fn const_fn<T, U, N: Nat>(x: T, _y: U) -> T {
            x
        }

        /// 组合函数
        pub fn compose<T, U, V, N: Nat>(
            f: impl Fn(U) -> V,
            g: impl Fn(T) -> U,
        ) -> impl Fn(T) -> V {
            move |x| f(g(x))
        }

        /// 依赖类型的 flip
        pub fn flip<T, U, V, N: Nat>(
            f: impl Fn(T, U) -> V,
        ) -> impl Fn(U, T) -> V {
            move |y, x| f(x, y)
        }
    }

    // ==================== 依赖类型示例 ====================

    /// 安全的列表索引（依赖类型）
    pub struct SafeIndex<T, N: Nat> {
        pub value: T,
        pub index: N,
        pub list: Vec<T>,
    }

    impl<T, N> SafeIndex<T, N>
    where
        N: Nat + Clone + PartialEq<usize>,
    {
        pub fn new(value: T, index: N, list: &[T]) -> Option<SafeIndex<T, N>> {
            let idx = index.to_usize()?;
            if idx < list.len() && list[idx] == value {
                Some(SafeIndex {
                    value,
                    index,
                    list: list.to_vec(),
                })
            } else {
                None
            }
        }
    }

    /// 依赖类型的向量（长度在类型中）
    pub struct Vector<T, N: Nat> {
        pub elements: Vec<T>,
        pub length: N,
    }

    impl<T, N: Nat> Vector<T, N> {
        pub fn nil() -> Vector<T, Nat::Z> {
            Vector {
                elements: Vec::new(),
                length: Nat::Z,
            }
        }

        pub fn cons(head: T, tail: &Vector<T, N>) -> Vector<T, Nat::S<N>> {
            let mut new_elements = Vec::with_capacity(tail.elements.len() + 1);
            new_elements.push(head);
            new_elements.extend_from_slice(&tail.elements);
            
            Vector {
                elements: new_elements,
                length: Nat::S(Box::new(tail.length.clone())),
            }
        }

        pub fn length(&self) -> N {
            self.length.clone()
        }

        pub fn head(&self) -> Option<&T> {
            self.elements.first()
        }

        pub fn tail(&self) -> Option<&Vector<T, N>> {
            if self.elements.len() > 0 {
                let tail_elements = self.elements[1..].to_vec();
                let tail_length = self.length.sub(&Nat::one())?;
                Some(&Vector {
                    elements: tail_elements,
                    length: tail_length,
                })
            } else {
                None
            }
        }

        pub fn index(&self, i: N) -> Option<&T>
        where
            N: PartialEq<usize> + Clone,
        {
            let idx = i.to_usize()?;
            self.elements.get(idx)
        }

        pub fn append(&self, other: &Vector<T, M>) -> Vector<T, Nat::Add<N, M>>
        where
            N: Nat + Clone,
            M: Nat,
        {
            let mut new_elements = self.elements.clone();
            new_elements.extend_from_slice(&other.elements);
            
            Vector {
                elements: new_elements,
                length: self.length.add(&other.length),
            }
        }

        pub fn reverse(&self) -> Vector<T, N>
        where
            N: Nat + Clone,
        {
            let mut new_elements = self.elements.clone();
            new_elements.reverse();
            Vector {
                elements: new_elements,
                length: self.length.clone(),
            }
        }

        pub fn map<U, F: Fn(&T) -> U>(&self, f: F) -> Vector<U, N> {
            let new_elements: Vec<U> = self.elements.iter().map(f).collect();
            Vector {
                elements: new_elements,
                length: self.length.clone(),
            }
        }

        pub fn foldl<U, F: Fn(U, &T) -> U>(&self, init: U, f: F) -> U {
            let mut result = init;
            for elem in &self.elements {
                result = f(result, elem);
            }
            result
        }
    }

    // ==================== 类型级函数 ====================

    pub mod type_level {
        use super::{Nat, TypeNat};

        /// 类型的加法
        pub type Add<A, B> = 
            where
                A: Nat,
                B: Nat;

        /// 类型的乘法
        pub type Mul<A, B> =
            where
                A: Nat,
                B: Nat;

        /// 类型的减法
        pub type Sub<A, B> =
            where
                A: Nat,
                B: Nat;

        // 辅助 trait
        pub trait NatAdd<A: Nat> {
            type Output: Nat;
        }

        pub trait NatSub<A: Nat> {
            type Output: Nat;
        }

        pub trait NatMul<A: Nat> {
            type Output: Nat;
        }

        // 为 Nat 实现加法
        impl<N: Nat> NatAdd<Nat::Z> for N {
            type Output = N;
        }

        impl<N: Nat> NatAdd<Nat::S<M>> for N
        where
            M: Nat,
        {
            type Output = Nat::S<N::Output>;
        }

        // 为 TypeNat 实现加法
        impl TypeNat {
            pub fn add_type<A: Nat + From<TypeNat>>(&self, other: &A) -> A {
                A::from(TypeNat(self.0 + other.to_usize().map(|n| n as i128).unwrap_or(0)))
            }
        }
    }

    // ==================== 异构列表 ====================

    #[derive(Debug, Clone)]
    pub struct HList<H, T: HListLike> {
        pub head: H,
        pub tail: T,
    }

    pub trait HListLike {
        fn len(&self) -> usize;
    }

    impl HListLike for () {
        fn len(&self) -> usize {
            0
        }
    }

    impl<H, T: HListLike> HListLike for HList<H, T> {
        fn len(&self) -> usize {
            1 + self.tail.len()
        }
    }

    // ====================  GADT 支持 ====================

    #[derive(Debug, Clone)]
    pub enum GADT<T> {
        Int(i32),
        Float(f64),
        Bool(bool),
        Char(char),
        String(String),
    }

    impl<T> GADT<T> {
        pub fn as_int(&self) -> Option<i32> {
            match self {
                GADT::Int(n) => Some(*n),
                _ => None,
            }
        }

        pub fn as_float(&self) -> Option<f64> {
            match self {
                GADT::Float(f) => Some(*f),
                _ => None,
            }
        }
    }
}
