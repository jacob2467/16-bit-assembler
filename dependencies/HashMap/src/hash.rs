pub trait JacobHash {
    fn jacob_hash(&self) -> usize {
        0
    }
}

impl JacobHash for &str {
    fn jacob_hash(&self) -> usize {
        let mut hashed: usize = 13;
        for c in self.chars() {
            hashed = hashed.wrapping_add(c as usize);
            hashed = hashed.wrapping_mul(c as usize);
        };
        hashed
    }
}

impl JacobHash for &String {
    fn jacob_hash(&self) -> usize {
        self.as_str().jacob_hash()
    }
}

impl JacobHash for String {
    fn jacob_hash(&self) -> usize {
        let mut hashed: usize = 13;
        for c in self.chars() {
            hashed = hashed.wrapping_add(c as usize);
            hashed = hashed.wrapping_mul(c as usize);
        };
        hashed
    }
}

impl JacobHash for i32 {
    fn jacob_hash(&self) -> usize {
        let s = self.to_string();
        let mut hashed: usize = 42069;
        for c in s.chars() {
            hashed = hashed.wrapping_add(c as usize);
            hashed = hashed.wrapping_mul(c as usize);
        };
        hashed
    }
}