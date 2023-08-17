use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    ptr,
    sync::Arc,
};

use crate::{array::*, value::Value, Uiua, UiuaResult};

impl Value {
    pub fn deshape(&mut self) {
        self.generic_mut(
            Array::deshape,
            Array::deshape,
            Array::deshape,
            Array::deshape,
        )
    }
    pub fn parse_num(&self, env: &Uiua) -> UiuaResult<Self> {
        Ok(self
            .as_string(env, "Parsed array must be a string")?
            .parse::<f64>()
            .map_err(|e| env.error(format!("Cannot parse into number: {}", e)))?
            .into())
    }
}

impl<T: ArrayValue> Array<T> {
    pub fn deshape(&mut self) {
        self.shape = vec![self.flat_len()];
    }
}

impl Value {
    pub fn range(&self, env: &Uiua) -> UiuaResult<Self> {
        let mut shape = self.as_naturals(
            env,
            "Range max should be a single natural number \
            or a list of natural numbers",
        )?;
        let data = range(&shape);
        if shape.len() > 1 {
            shape.push(shape.len());
        }
        let array = Array::new(shape, data.into());
        Ok(array.into())
    }
}

fn range(shape: &[usize]) -> Vec<f64> {
    if shape.is_empty() {
        return vec![0.0];
    }
    if shape.contains(&0) {
        return Vec::new();
    }
    let len = shape.len() * shape.iter().product::<usize>();
    let mut data: Vec<f64> = Vec::with_capacity(len);
    let mut curr = vec![0; shape.len()];
    loop {
        for d in &curr {
            data.push(*d as f64);
        }
        let mut i = shape.len() - 1;
        loop {
            curr[i] += 1;
            if curr[i] == shape[i] {
                curr[i] = 0;
                if i == 0 {
                    return data;
                }
                i -= 1;
            } else {
                break;
            }
        }
    }
}

impl Value {
    pub fn first(self, env: &Uiua) -> UiuaResult<Self> {
        Ok(match self {
            Self::Num(array) => array.first(env)?.into(),
            Self::Byte(array) => array.first(env)?.into(),
            Self::Char(array) => array.first(env)?.into(),
            Self::Func(array) => array.first(env)?.into(),
        })
    }
    pub fn last(self, env: &Uiua) -> UiuaResult<Self> {
        Ok(match self {
            Self::Num(array) => array.last(env)?.into(),
            Self::Byte(array) => array.last(env)?.into(),
            Self::Char(array) => array.last(env)?.into(),
            Self::Func(array) => array.last(env)?.into(),
        })
    }
}

impl<T: ArrayValue> Array<T> {
    pub fn first(mut self, env: &Uiua) -> UiuaResult<Self> {
        if self.rank() == 0 {
            return Err(env.error("Cannot take first of a scalar"));
        }
        if self.shape[0] == 0 {
            return Err(env.error("Cannot take first of an empty array"));
        }
        let row_len = self.row_len();
        self.shape.remove(0);
        self.data.truncate(row_len);
        Ok(self)
    }
    pub fn last(mut self, env: &Uiua) -> UiuaResult<Self> {
        if self.rank() == 0 {
            return Err(env.error("Cannot take last of a scalar"));
        }
        let row_len = self.row_len();
        self.shape.remove(0);
        let prefix_len = self.data.len() - row_len;
        self.data = self.data.into_iter().skip(prefix_len).collect();
        Ok(self)
    }
}

impl Value {
    pub fn reverse(&mut self) {
        self.generic_mut(
            Array::reverse,
            Array::reverse,
            Array::reverse,
            Array::reverse,
        )
    }
}

impl<T: ArrayValue> Array<T> {
    pub fn reverse(&mut self) {
        if self.shape.is_empty() {
            return;
        }
        let row_count = self.row_count();
        let row_len = self.row_len();
        for i in 0..row_count / 2 {
            let left = i * row_len;
            let right = (row_count - i - 1) * row_len;
            let left = &mut self.data[left] as *mut T;
            let right = &mut self.data[right] as *mut T;
            unsafe {
                ptr::swap_nonoverlapping(left, right, row_len);
            }
        }
    }
}

impl Value {
    pub fn transpose(&mut self) {
        self.generic_mut(
            Array::transpose,
            Array::transpose,
            Array::transpose,
            Array::transpose,
        )
    }
    pub fn inv_transpose(&mut self) {
        self.generic_mut(
            Array::inv_transpose,
            Array::inv_transpose,
            Array::inv_transpose,
            Array::inv_transpose,
        )
    }
}

impl<T: ArrayValue> Array<T> {
    pub fn transpose(&mut self) {
        crate::profile_function!();
        if self.shape.len() < 2 {
            return;
        }
        if self.shape[0] == 0 {
            self.shape.rotate_left(1);
            return;
        }
        let mut temp = Vec::with_capacity(self.data.len());
        let row_len = self.row_len();
        let row_count = self.row_count();
        for j in 0..row_len {
            for i in 0..row_count {
                temp.push(self.data[i * row_len + j].clone());
            }
        }
        self.data = temp.into();
        self.shape.rotate_left(1);
    }
    pub fn inv_transpose(&mut self) {
        crate::profile_function!();
        if self.shape.len() < 2 {
            return;
        }
        if self.shape[0] == 0 {
            self.shape.rotate_right(1);
            return;
        }
        let mut temp = Vec::with_capacity(self.data.len());
        let col_len = *self.shape.last().unwrap();
        let col_count: usize = self.shape.iter().rev().skip(1).product();
        for j in 0..col_len {
            for i in 0..col_count {
                temp.push(self.data[i * col_len + j].clone());
            }
        }
        self.data = temp.into();
        self.shape.rotate_right(1);
    }
}

impl Value {
    pub fn grade(&self, env: &Uiua) -> UiuaResult<Self> {
        Ok(Self::from_iter(match self {
            Self::Num(array) => array.grade(env)?,
            Self::Byte(array) => array.grade(env)?,
            Self::Char(array) => array.grade(env)?,
            Self::Func(array) => array.grade(env)?,
        }))
    }
    pub fn classify(&self, env: &Uiua) -> UiuaResult<Self> {
        Ok(Self::from_iter(match self {
            Self::Num(array) => array.classify(env)?,
            Self::Byte(array) => array.classify(env)?,
            Self::Char(array) => array.classify(env)?,
            Self::Func(array) => array.classify(env)?,
        }))
    }
    pub fn deduplicate(&mut self) {
        self.generic_mut(
            Array::deduplicate,
            Array::deduplicate,
            Array::deduplicate,
            Array::deduplicate,
        )
    }
}

impl<T: ArrayValue> Array<T> {
    pub fn grade(&self, env: &Uiua) -> UiuaResult<Vec<usize>> {
        if self.rank() == 0 {
            return Err(env.error("Cannot grade a rank-0 array"));
        }
        let mut indices = (0..self.flat_len()).collect::<Vec<_>>();
        indices.sort_by(|&a, &b| {
            self.row_slice(a)
                .iter()
                .zip(self.row_slice(b))
                .map(|(a, b)| a.array_cmp(b))
                .find(|x| x != &Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        });
        Ok(indices)
    }
    pub fn classify(&self, env: &Uiua) -> UiuaResult<Vec<usize>> {
        if self.rank() == 0 {
            return Err(env.error("Cannot classify a rank-0 array"));
        }
        let mut classes = BTreeMap::new();
        let mut classified = Vec::with_capacity(self.row_count());
        for row in self.rows() {
            let new_class = classes.len();
            let class = *classes.entry(row).or_insert(new_class);
            classified.push(class);
        }
        Ok(classified)
    }
    pub fn deduplicate(&mut self) {
        if self.rank() == 0 {
            return;
        }
        let mut deduped = Vec::new();
        let mut seen = BTreeSet::new();
        let mut new_len = 0;
        for row in self.rows() {
            if seen.insert(row.clone()) {
                deduped.extend_from_slice(&row.data);
                new_len += 1;
            }
        }
        self.data = deduped.into();
        self.shape[0] = new_len;
    }
}

impl Value {
    pub fn invert(&self, env: &Uiua) -> UiuaResult<Self> {
        Ok(match self {
            Self::Func(fs) => {
                let mut invs = Vec::with_capacity(fs.row_count());
                for f in &fs.data {
                    invs.push(
                        f.inverse()
                            .ok_or_else(|| env.error("No inverse found"))?
                            .into(),
                    );
                }
                Self::Func((fs.shape.clone(), invs).into())
            }
            v => return Err(env.error(format!("Cannot invert {}", v.type_name()))),
        })
    }
    pub fn under(self, env: &Uiua) -> UiuaResult<(Self, Self)> {
        Ok(match self {
            Self::Func(fs) => {
                let mut befores = Vec::with_capacity(fs.row_count());
                let mut afters = Vec::with_capacity(fs.row_count());
                for f in fs.data {
                    let f = Arc::try_unwrap(f).unwrap_or_else(|f| (*f).clone());
                    let (before, after) = f.under().ok_or_else(|| env.error("No inverse found"))?;
                    befores.push(before.into());
                    afters.push(after.into());
                }
                (
                    Self::Func((fs.shape.clone(), befores).into()),
                    Self::Func((fs.shape.clone(), afters).into()),
                )
            }
            v => return Err(env.error(format!("Cannot invert {}", v.type_name()))),
        })
    }
}

impl Value {
    pub fn bits(&self, env: &Uiua) -> UiuaResult<Array<u8>> {
        match self {
            Value::Byte(n) => n.convert_ref().bits(env),
            Value::Num(n) => n.bits(env),
            _ => Err(env.error("Argument to bits must be an array of natural numbers")),
        }
    }
    pub fn inverse_bits(&self, env: &Uiua) -> UiuaResult<Array<f64>> {
        match self {
            Value::Byte(n) => n.inverse_bits(env),
            Value::Num(n) => n.convert_ref_with(|n| n as u8).inverse_bits(env),
            _ => Err(env.error("Argument to inverse_bits must be an array of naturals")),
        }
    }
}

impl Array<f64> {
    pub fn bits(&self, env: &Uiua) -> UiuaResult<Array<u8>> {
        let mut nats = Vec::new();
        for &n in &self.data {
            if n.fract() != 0.0 {
                return Err(env.error("Array must be a list of naturals"));
            }
            nats.push(n as u128);
        }
        let mut max = if let Some(max) = nats.iter().max() {
            *max
        } else {
            let mut shape = self.shape.clone();
            shape.push(0);
            return Ok((shape, Vec::new()).into());
        };
        let mut max_bits = 0;
        while max != 0 {
            max_bits += 1;
            max >>= 1;
        }
        let mut new_data = Vec::with_capacity(self.data.len() * max_bits);
        // Big endian
        for n in nats {
            for i in 0..max_bits {
                new_data.push(u8::from(n & (1 << i) != 0));
            }
        }
        let mut shape = self.shape.clone();
        shape.push(max_bits);
        let arr = Array::new(shape, new_data.into());
        arr.validate_shape();
        Ok(arr)
    }
}

impl Array<u8> {
    pub fn inverse_bits(&self, env: &Uiua) -> UiuaResult<Array<f64>> {
        let mut bools = Vec::with_capacity(self.data.len());
        for &b in &self.data {
            if b > 1 {
                return Err(env.error("Array must be a list of booleans"));
            }
            bools.push(b != 0);
        }
        if self.rank() == 0 {
            return Ok(Array::from(bools[0] as u8 as f64));
        }
        let mut shape = self.shape.clone();
        let bit_string_len = shape.pop().unwrap();
        let mut new_data = Vec::with_capacity(self.data.len() / bit_string_len);
        // Big endian
        for bits in bools.chunks_exact(bit_string_len) {
            let mut n = 0;
            for (i, b) in bits.iter().enumerate() {
                if *b {
                    n |= 1 << i;
                }
            }
            new_data.push(n as f64);
        }
        let arr = Array::new(shape, new_data.into());
        arr.validate_shape();
        Ok(arr)
    }
}
