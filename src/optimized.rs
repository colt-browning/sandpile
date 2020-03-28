use super::*;

impl GridSandpile {
	pub(super) fn delta00_infinite_optimized(&mut self) {
		assert_eq!(self.grid_type, GridType::Infinite(0, 0));
		assert_eq!(self.grid.len(), 1);
		assert_eq!(self.grid[0].len(), 1);
		let mut excessive = vec![(0, 0)];
		let mut ex2;
		let mut count = 0;
		while !excessive.is_empty() {
			ex2 = Vec::new();
			for (i, j) in excessive {
				let d = self.grid[i][j] / self.neighbourhood.neighbours();
				if d == 0 {
					continue;
				}
				if i + 1 == self.grid.len() {
					for row in self.grid.iter_mut() {
						row.push(0);
					}
					self.grid.push(vec![0; self.grid[0].len()]);
				}
				self.grid[i][j] %= self.neighbourhood.neighbours();
				count += match (i, j) {
					(0, 0) => 1,
					(_, 0) => 4,
					(i, j) if i == j => 4,
					_ => 8,
				} * d as u64;
				let mut topple_to: Vec<_> = match (i, j) {
					(0, 0) => vec![(1, 0)],
					(1, 0) => vec![(2, 0), (0, 0), (1, 1), (1, 1)],
					(1, 1) => vec![(1, 0), (1, 0), (2, 1)],
					(i, 0) => vec![(i-1, 0), (i, 1), (i+1, 0)],
					(2, 1) => vec![(1, 1), (1, 1), (2, 0), (2, 0), (2, 2), (2, 2), (3, 1)],
					(i, j) if i == j => vec![(i, j-1), (i+1, j)],
					(i, 1) => vec![(i, 0), (i, 0), (i-1, 1), (i, 2), (i+1, 1)],
					(i, j) if i == j+1 => vec![(j, j), (j, j), (i, i), (i, i), (i+1, j), (i, j-1)],
					(i, j) => vec![(i-1, j), (i, j+1), (i+1, j), (i, j-1)],
				};
				if self.neighbourhood == Neighbourhood::Moore {
					let mut t2: Vec<_> = match (i, j) {
						(0, 0) => vec![(1, 1)],
						(1, 0) => vec![(2, 1), (1, 0), (1, 0)],
						(1, 1) => vec![(0, 0), (2, 0), (2, 0), (2, 2)],
						(2, 0) => vec![(1, 1), (1, 1), (3, 1)],
						(2, 1) => vec![(1, 0), (1, 0), (3, 0), (3, 0), (2, 1), (3, 2)],
						(i, j) if i == j => vec![(i-1, j-1), (i+1, j-1), (i+1, j+1)],
						(i, 0) => vec![(i-1, 1), (i+1, 1)],
						(3, 1) => vec![(2, 0), (2, 0), (4, 0), (4, 0), (2, 2), (2, 2), (4, 2)],
						(i, j) if i == j+1 => vec![(i-1, j-1), (i+1, j-1), (i+1, j+1), (i, j)],
						(i, j) if i == j+2 => vec![(i-1, j-1), (i+1, j-1), (i+1, j+1), (i-1, j+1), (i-1, j+1)],
						(i, j) => vec![(i-1, j-1), (i+1, j-1), (i+1, j+1), (i-1, j+1)],
					};
					topple_to.append(&mut t2);
				}
				for (ti, tj) in topple_to {
					self.grid[ti][tj] += if (ti, tj) == (0, 0) {4*d} else {d};
					if let Some(p) = ex2.last() {
						if *p == (ti, tj) {
							continue
						}
					}
					if self.grid[ti][tj] >= self.neighbourhood.neighbours() {
						ex2.push((ti, tj));
					}
				}
			}
			excessive = ex2;
		}
		self.last_topple = count;
		self.grid_type = GridType::Infinite(self.grid.len()-1, self.grid.len()-1);
		for i in 1..self.grid.len() {
			for j in 0..i {
				self.grid[j][i] = self.grid[i][j];
			}
		}
		for row in &mut self.grid {
			let mut mirrow: Vec<_> = row.iter().skip(1).rev().map(|x| *x).collect();
			mirrow.append(row);
			*row = mirrow;
		}
		let mut mirrid: Vec<_> = self.grid.iter().skip(1).rev().map(|x| x.clone()).collect();
		mirrid.append(&mut self.grid);
		self.grid = mirrid;
	}
}

impl<'a> FiniteGridSandpile<'a> {
	pub(super) fn neutral_rect_vn_es_optimized(x: usize) -> GridSandpile { // es = even square
		let t = 6;
		let mut symmetric_grid: Vec<_> = (0..x).map(|i| vec![t; i+1]).collect();
		topple_rect_vn_es_optimized(&mut symmetric_grid);
		for row in &mut symmetric_grid {
			for el in row {
				*el = t - *el;
			}
		}
		topple_rect_vn_es_optimized(&mut symmetric_grid);
		for i in 0..x {
			for j in i+1..x {
				let sc = symmetric_grid[j][i];
				symmetric_grid[i].push(sc);
			}
		}
		let mut grid = Vec::new();
		while let Some(mut s_row) = symmetric_grid.pop() {
			let mut row: Vec<_> = s_row.clone().into_iter().rev().collect();
			row.append(&mut s_row);
			grid.push(row);
		}
		for i in 1..=x {
			grid.push(grid[x-i].clone());
		}
		GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::VonNeumann, grid).unwrap()
	}
}

pub(super) fn topple_rect_vn_es_optimized(grid: &mut Vec<Vec<Cell>>) {
	let x = grid.len();
	assert!(x > 2);
	let mut ex_table = Vec::new();
	for i in 0..x {
		ex_table.push(vec![true; i+1]);
	}
	let mut use_vec = false;
	let lim = x + x*x/50;
	let mut excessive = Vec::new();
	for i in 0..x {
		assert_eq!(i+1, grid[i].len());
		for j in 0..grid[i].len() {
			excessive.push((i, j));
		}
	}
	let mut ex2;
	while !use_vec || !excessive.is_empty() {
		let use_vec_now = use_vec;
		use_vec = true;
		ex2 = Vec::with_capacity(lim+1);
		let mut i = 0;
		let mut j = 0;
		loop {
			if use_vec_now {
				if let Some((ix, jx)) = excessive.pop() {
					i = ix;
					j = jx;
				} else {
					break
				}
			} else {
				if !ex_table[i][j] {
					j += 1;
					if j > i {
						j = 0;
						i += 1;
						if i == x {
							break
						}
					}
					continue
				}
			}
			ex_table[i][j] = false;
			let d = grid[i][j] / 4;
			if d == 0 {
				continue;
			}
			grid[i][j] %= 4;
			let topple_to: Vec<_> = match (i, j) {
				(0, 0) => vec![(0, 0), (0, 0), (1, 0)],
				(1, 0) => vec![(1, 0), (2, 0), (0, 0), (0, 0), (1, 1), (1, 1)],
				(i, j) if i == x-1 && j == x-1 => vec![(x-1, x-2)],
				(i, j) if i == j => vec![(i+1, i), (i, i-1)],
				(i, 0) if i == x-1 => vec![(x-1, 0), (x-2, 0), (x-1, 1)],
				(i, 0) => vec![(i, 0), (i-1, 0), (i+1, 0), (i, 1)],
				(i, j) if i == x-1 && j == x-2 => vec![(x-1, x-1), (x-1, x-1), (x-2, x-2), (x-2, x-2), (x-1, x-3)],
				(i, j) if j == i-1 => vec![(i-1, j), (i-1, j), (i, j+1), (i, j+1), (i, j-1), (i+1, j)],
				(i, j) if i == x-1 => vec![(x-1, j-1), (x-1, j+1), (x-2, j)],
				(i, j) => vec![(i-1, j), (i+1, j), (i, j-1), (i, j+1)],
			};
			for (ti, tj) in topple_to {
				grid[ti][tj] += d;
				if let Some(p) = ex2.last() {
					if *p == (ti, tj) {
						continue
					}
				}
				if grid[ti][tj] >= 4 {
					ex_table[ti][tj] = true;
					if use_vec {
						ex2.push((ti, tj));
						if ex2.len() >= lim {
							ex2.clear();
							use_vec = false;
						}
					}
				}
			}
			if !use_vec_now {
				j += 1;
				if j > i {
					j = 0;
					i += 1;
					if i == x {
						break
					}
				}
			}
		}
		excessive = ex2;
	}
}
