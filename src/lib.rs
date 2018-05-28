extern crate repng;

use std::{
	collections::HashSet,
	io,
	fs::File,
	fmt,
};

const ADD_ERR_MSG: &str = "Attempt to add sandpiles on grids of different sizes.";

pub trait Sandpile: PartialEq + Clone {
	fn topple(&mut self) -> u64;
	fn neutral(usize, usize) -> Self;
	fn add(&mut self, &Self) -> Result<(), &str>;
	fn order(&self) -> u64
	{
		let mut a = self.clone();
		a.add(self).unwrap();
		let mut count = 1;
		while a != *self {
			a.add(self).unwrap();
			count += 1;
		}
		count
	}
	fn into_graph(self) -> Vec<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FiniteGrid {
	graph: Vec<Vec<u8>>,
}

impl FiniteGrid {
	fn from_graph(graph: Vec<Vec<u8>>) -> Result<FiniteGrid, &'static str> {
		if graph.is_empty() {
			return Err("Empty graph");
		}
		let l = graph[0].len();
		if l == 0 {
			return Err("Empty first row");
		}
		for row in &graph {
			if row.len() != l {
				return Err("Rows of unequal lengths");
			}
		}
		Ok(FiniteGrid {
			graph,
		})
	}
}

impl fmt::Display for FiniteGrid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", fmt_graph(&self.graph))
	}
}

impl Sandpile for FiniteGrid {
	fn topple(&mut self) -> u64 {
		let mut excessive = HashSet::new();
		let mut ex2;
		for i in 0..self.graph.len() {
			for j in 0..self.graph[i].len() {
				if self.graph[i][j] >= 4 {
					excessive.insert((i, j));
				}
			}
		}
		let mut count = 0;
		while !excessive.is_empty() {
			ex2 = HashSet::new();
			for c in excessive.drain() {
				let (i, j) = c;
				let d = self.graph[i][j] / 4;
				self.graph[i][j] %= 4;
				count += d as u64;
				if i > 0 {
					self.graph[i-1][j] += d;
					if self.graph[i-1][j] >= 4 {
						ex2.insert((i-1, j));
					}
				}
				if j > 0 {
					self.graph[i][j-1] += d;
					if self.graph[i][j-1] >= 4 {
						ex2.insert((i, j-1));
					}
				}
				if i < self.graph.len()-1 {
					self.graph[i+1][j] += d;
					if self.graph[i+1][j] >= 4 {
						ex2.insert((i+1, j));
					}
				}
				if j < self.graph[i].len()-1 {
					self.graph[i][j+1] += d;
					if self.graph[i][j+1] >= 4 {
						ex2.insert((i, j+1));
					}
				}
			}
			excessive = ex2;
		}
		count
	}
	
	fn add(&mut self, p: &FiniteGrid) -> Result<(), &str> {
		if p.graph.len() != self.graph.len() || p.graph[0].len() != self.graph[0].len() {
			return Err(ADD_ERR_MSG);
		}
		for i in 0..self.graph.len() {
			for j in 0..self.graph[0].len() {
				self.graph[i][j] += p.graph[i][j];
			}
		}
		self.topple();
		Ok(())
	}
	
	fn neutral(x: usize, y: usize) -> FiniteGrid {
	// Proposition 6.36 of http://people.reed.edu/~davidp/divisors_and_sandpiles/
		let mut grid = FiniteGrid::from_graph(vec![vec![6; x]; y]).unwrap(); // TODO: ?
		grid.topple();
		for mut row in grid.graph.iter_mut() {
			for mut el in row {
				*el = 6 - *el;
			}
		}
		grid.topple();
		grid
	}

	fn into_graph(self) -> Vec<Vec<u8>> {
		self.graph
	}
}


#[derive(Debug, Clone, PartialEq)]
pub struct ToroidalGrid {
	graph: Vec<Vec<u8>>,
}

impl ToroidalGrid {
	fn from_graph(graph: Vec<Vec<u8>>) -> Result<ToroidalGrid, &'static str> {
		if graph.is_empty() {
			return Err("Empty graph");
		}
		let l = graph[0].len();
		if l == 0 {
			return Err("Empty first row");
		}
		for row in &graph {
			if row.len() != l {
				return Err("Rows of unequal lengths");
			}
		}
		let mut g = ToroidalGrid {
			graph,
		};
		g.graph[0][0] = 0;
		Ok(g)
	}
}

impl fmt::Display for ToroidalGrid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", fmt_graph(&self.graph))
	}
}

impl Sandpile for ToroidalGrid {
	fn topple(&mut self) -> u64 {
		let mut excessive = HashSet::new();
		let mut ex2;
		for i in 0..self.graph.len() {
			for j in 0..self.graph[i].len() {
				if self.graph[i][j] >= 4 {
					excessive.insert((i, j));
				}
			}
		}
		let mut count = 0;
		while !excessive.is_empty() {
			ex2 = HashSet::new();
			for c in excessive.drain() {
				let (i, j) = c;
				let d = self.graph[i][j] / 4;
				self.graph[i][j] %= 4;
				count += d as u64;
				let i1 = if i > 0 {i-1} else {self.graph.len()-1};
				if !(i1 == 0 && j == 0) {
					self.graph[i1][j] += d;
					if self.graph[i1][j] >= 4 {
						ex2.insert((i1, j));
					}
				}
				let j1 = if j > 0 {j-1} else {self.graph[0].len()-1};
				if !(i == 0 && j1 == 0) {
					self.graph[i][j1] += d;
					if self.graph[i][j1] >= 4 {
						ex2.insert((i, j1));
					}
				}
				let i1 = if i < self.graph.len()-1 {i+1} else {0};
				if !(i1 == 0 && j == 0) {
					self.graph[i1][j] += d;
					if self.graph[i1][j] >= 4 {
						ex2.insert((i1, j));
					}
				}
				let j1 = if j < self.graph[i].len()-1 {j+1} else {0};
				if !(i == 0 && j1 == 0) {
					self.graph[i][j1] += d;
					if self.graph[i][j1] >= 4 {
						ex2.insert((i, j1));
					}
				}
			}
			excessive = ex2;
		}
		count
	}
	
	fn add(&mut self, p: &ToroidalGrid) -> Result<(), &str> {
		if p.graph.len() != self.graph.len() || p.graph[0].len() != self.graph[0].len() {
			return Err(ADD_ERR_MSG);
		}
		for i in 0..self.graph.len() {
			for j in 0..self.graph[0].len() {
				self.graph[i][j] += p.graph[i][j];
			}
		}
		self.topple();
		Ok(())
	}
	
	fn neutral(x: usize, y: usize) -> ToroidalGrid {
	// Proposition 6.36 of http://people.reed.edu/~davidp/divisors_and_sandpiles/
		let mut grid = ToroidalGrid::from_graph(vec![vec![6; x]; y]).unwrap(); // TODO: ?
		grid.topple();
		for mut row in grid.graph.iter_mut() {
			for mut el in row {
				*el = 6 - *el;
			}
		}
		grid.graph[0][0] = 0;
		grid.topple();
		grid
	}

	fn into_graph(self) -> Vec<Vec<u8>> {
		self.graph
	}
}

pub fn png(graph: &Vec<Vec<u8>>, fname: &str) -> Result<(), io::Error> {
	let colors = [
		[0, 0, 0, 255],
		[64, 128, 0, 255],
		[118, 8, 170, 255],
		[255, 214, 0, 255],
	];
	let mut pixels = vec![0; graph.len() * graph[0].len() * 4];
	let mut p = 0;
	for row in graph.iter() {
		for el in row {
			pixels[p..p+4].copy_from_slice(&colors[*el as usize]);
			p += 4;
		}
	}
	repng::encode(File::create(fname)?, graph[0].len() as u32, graph.len() as u32, &pixels)
}

fn fmt_graph(graph: &Vec<Vec<u8>>) -> String {
	let vis = [" ", ".", ":", "&"];
	let mut s = String::new();
	for row in graph.iter() {
		for el in row {
			s += vis[*el as usize];
		}
		s += "\n";
	}
	s
}
