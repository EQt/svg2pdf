trait ToArray {
    fn to_arr(&self) -> [f64; 6];
}

impl ToArray for Transform {
    fn to_arr(&self) -> [f64; 6] {
        [
            self.a, 
            self.b, 
            self.c, 
            self.d, 
            self.e, 
            self.f,
        ]
    }
}
