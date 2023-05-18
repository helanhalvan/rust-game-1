use std::fmt::Debug;

use crate::hexgrid::XYCont;

#[derive(Debug, Clone)]
pub(crate) struct Matrix<T> {
    size_y: usize,
    data: Vec<T>,
}

pub(crate) fn new<T: Clone>(size_x: usize, size_y: usize, s: Vec<Vec<T>>) -> Matrix<T> {
    let mut data = Vec::with_capacity(size_x * size_y);
    for x in 0..size_x {
        for y in 0..size_y {
            let index = (x * size_y) + y;
            data.insert(index, s[x][y].clone())
        }
    }
    Matrix {
        size_y: size_y,
        data: data,
    }
}

pub(crate) fn set<T: Clone>(m: &mut Matrix<T>, i: T, XYCont { x, y }: XYCont<usize>) {
    let index = (x * m.size_y) + y;
    m.data[index] = i.clone();
}

pub(crate) fn get<T: Clone + std::cmp::PartialEq + Debug>(
    m: &Matrix<T>,
    XYCont { x, y }: XYCont<usize>,
) -> Option<&T> {
    let index = (x * m.size_y) + y;
    m.data.get(index)
}
