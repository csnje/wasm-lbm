pub mod parameters;

use crate::linear_algebra::VectorOps;

use itertools::izip;

/// Boundary schemes.
#[derive(Clone, Copy)]
pub enum BoundaryScheme {
    Inflow,
    Outflow,
    Periodic,
    BounceBack,
    SpecularReflection,
}

/// Lattice parameters,
#[derive(Clone, Copy)]
struct LatticeParameters<const N: usize> {
    lattice_vector: [isize; N],
    weight: f32,
}

impl<const N: usize> Default for LatticeParameters<N> {
    fn default() -> Self {
        Self {
            lattice_vector: [0; N],
            weight: 0.0,
        }
    }
}

/// Algorithm values.
#[derive(Clone, Copy)]
struct AlgorithmValues<const N: usize, const B: usize> {
    distributions: [f32; B],
    collision_distributions: [f32; B],
    density: f32,
    velocity_vector: [f32; N],
}

/// Implementation of the [Lattice Boltzmann method (LBM)](https://en.wikipedia.org/wiki/Lattice_Boltzmann_methods) for the
/// [Bhatnagar–Gross–Krook (BGK) operator](https://en.wikipedia.org/wiki/Bhatnagar%E2%80%93Gross%E2%80%93Krook_operator)
/// model for relaxation.
pub struct Lbgk<const N: usize, const B: usize> {
    lattice_parameters: [LatticeParameters<N>; B],
    sound_speed_squared: f32,
    size: [usize; N],
    boundary_schemes: [[BoundaryScheme; 2]; N],
    source_algorithm_values: AlgorithmValues<N, B>,
    // note: flat vectors reduce cache loads
    algorithm_values: Vec<AlgorithmValues<N, B>>,
    object: Vec<bool>,
}

impl Lbgk<2, 9> {
    /// Create `Lbgk` for the D2Q9 parameters.
    pub fn new_d2q9(
        size: &[usize; 2],
        boundary_schemes: &[[BoundaryScheme; 2]; 2],
        density: f32,
        velocity_vector: &[f32; 2],
    ) -> Self {
        let mut lattice_parameters = [LatticeParameters::default(); 9];
        for (lattice_parameters, c, w) in izip!(
            &mut lattice_parameters,
            parameters::d2q9::C,
            parameters::d2q9::W
        ) {
            lattice_parameters.lattice_vector = c;
            lattice_parameters.weight = w;
        }

        let sound_speed_squared = parameters::d2q9::CS2;

        let distributions = Self::equilibrium_distributions(
            &lattice_parameters,
            sound_speed_squared,
            density,
            velocity_vector,
        );

        let source_algorithm_values = AlgorithmValues::<2, 9> {
            distributions,
            collision_distributions: [0.0; 9],
            density,
            velocity_vector: *velocity_vector,
        };

        let len = size.iter().product();

        Self {
            lattice_parameters,
            sound_speed_squared,
            size: *size,
            boundary_schemes: *boundary_schemes,
            source_algorithm_values,
            algorithm_values: vec![source_algorithm_values; len],
            object: vec![false; len],
        }
    }
}

impl<const N: usize, const B: usize> Lbgk<N, B> {
    /// Flat array index for a lattice position.
    fn index(&self, pos: &[usize; N]) -> usize {
        izip!(pos.iter().skip(1), self.size)
            .fold(
                (pos[0], self.size[0]),
                |(result, multiplier), (pos, size)| (result + multiplier * pos, multiplier * size),
            )
            .0
    }

    /// Advances position in particular dimensions.
    fn next_pos(&self, pos: &mut [usize; N], dims: &[bool; N]) -> bool {
        let mut result = false;
        for (pos, idx, size) in izip!(pos, *dims, self.size) {
            if idx {
                *pos += 1;
                if *pos < size {
                    result = true;
                    break;
                }
                *pos = 0;
            }
        }
        result
    }

    /// Density at lattice position.
    pub fn density(&self, pos: &[usize; N]) -> f32 {
        self.algorithm_values[self.index(pos)].density
    }

    /// Velocity vector at lattice position.
    pub fn velocity_vector(&self, pos: &[usize; N]) -> [f32; N] {
        self.algorithm_values[self.index(pos)].velocity_vector
    }

    /// Velocity at lattice position.
    pub fn velocity(&self, pos: &[usize; N]) -> f32 {
        let u = &self.algorithm_values[self.index(pos)].velocity_vector;
        u.dot_product(u).sqrt()
    }

    /// [Vorticity](https://en.wikipedia.org/wiki/Vorticity) at lattice position.
    pub fn vorticity(&self, pos: &[usize; N]) -> f32 {
        match N {
            2 => {
                let mut result = 0.0;
                if izip!(pos, self.size).all(|(pos, size)| (1..size - 1).contains(pos)) {
                    let mut other_pos = [0; N];
                    (other_pos[0], other_pos[1]) = (pos[0] + 1, pos[1]);
                    result += self.velocity_vector(&other_pos)[1];
                    (other_pos[0], other_pos[1]) = (pos[0] - 1, pos[1]);
                    result -= self.velocity_vector(&other_pos)[1];
                    (other_pos[0], other_pos[1]) = (pos[0], pos[1] + 1);
                    result -= self.velocity_vector(&other_pos)[0];
                    (other_pos[0], other_pos[1]) = (pos[0], pos[1] - 1);
                    result += self.velocity_vector(&other_pos)[0];
                }
                result
            }
            3 => todo!(),
            _ => panic!(),
        }
    }

    /// Object at lattice position.
    pub fn object(&self, pos: &[usize; N]) -> bool {
        self.object[self.index(pos)]
    }

    /// Set object at lattice position.
    pub fn set_object(&mut self, pos: &[usize; N], val: bool) {
        let index = self.index(pos);
        self.object[index] = val;
    }

    /// Calculate relaxation time.
    pub fn relaxation_time(
        &self,
        velocity: f32,
        characteristic_length: f32,
        reynolds_number: f32,
    ) -> f32 {
        characteristic_length * velocity / (self.sound_speed_squared * reynolds_number) + 0.5
    }

    /// Perform iteration.
    pub fn iterate(&mut self, relaxation_time: f32) {
        self.collision_step(relaxation_time);
        self.streaming_step();
        self.calculate_derived();
        self.update_inflows_and_outflows();
    }

    /// Perform collision step of iteration.
    fn collision_step(&mut self, relaxation_time: f32) {
        let (mut pos, dims) = ([0; N], [true; N]);
        loop {
            let index = self.index(&pos);

            if self.object[index] {
                match self.next_pos(&mut pos, &dims) {
                    true => continue,
                    false => break,
                }
            }

            // calculate equilibrium distribution
            let algorithm_values = self.algorithm_values[index];
            let equilibrium_distributions = Self::equilibrium_distributions(
                &self.lattice_parameters,
                self.sound_speed_squared,
                algorithm_values.density,
                &algorithm_values.velocity_vector,
            );

            // calculate collision distribution
            let algorithm_values = &mut self.algorithm_values[index];
            for (f_c, f, f_eq) in izip!(
                &mut algorithm_values.collision_distributions,
                algorithm_values.distributions,
                equilibrium_distributions
            ) {
                *f_c = f - (f - f_eq) / relaxation_time;
            }

            if !self.next_pos(&mut pos, &dims) {
                break;
            }
        }
    }

    /// Perform streaming step of iteration.
    fn streaming_step(&mut self) {
        let (mut pos, dims) = ([0; N], [true; N]);
        loop {
            let index = self.index(&pos);

            if self.object[index] {
                match self.next_pos(&mut pos, &dims) {
                    true => continue,
                    false => break,
                }
            }

            for (i, lattice_parameters) in self.lattice_parameters.iter().enumerate() {
                let mut new_pos = [None; N];
                let mut new_lattice_vector = lattice_parameters.lattice_vector;
                let mut changed_lattice_vector = false;
                let mut bounce_back = false;
                for (new_pos, new_c, pos, c, size, boundary_schemes) in izip!(
                    &mut new_pos,
                    &mut new_lattice_vector,
                    pos,
                    lattice_parameters.lattice_vector,
                    self.size,
                    &self.boundary_schemes
                ) {
                    match pos as isize + c {
                        val if val < 0 => match boundary_schemes[0] {
                            BoundaryScheme::Periodic => *new_pos = Some(size - 1),
                            BoundaryScheme::BounceBack => bounce_back = true, // handled below
                            BoundaryScheme::SpecularReflection => {
                                *new_pos = Some(pos);
                                (*new_c, changed_lattice_vector) = (-c, true);
                            }
                            _ => {}
                        },
                        val if val >= size as isize => match boundary_schemes[1] {
                            BoundaryScheme::Periodic => *new_pos = Some(0),
                            BoundaryScheme::BounceBack => bounce_back = true, // handled below
                            BoundaryScheme::SpecularReflection => {
                                *new_pos = Some(pos);
                                (*new_c, changed_lattice_vector) = (-c, true);
                            }
                            _ => {}
                        },
                        val => *new_pos = Some(val as usize),
                    }
                }

                if !new_pos.contains(&None) {
                    let new_pos = new_pos.map(Option::unwrap);
                    let new_index = self.index(&new_pos);
                    if self.object[new_index] {
                        // TODO other boundary schemes
                        bounce_back = true;
                    }
                }

                if bounce_back {
                    for (pos_new, new_c, pos, c) in izip!(
                        &mut new_pos,
                        &mut new_lattice_vector,
                        pos,
                        lattice_parameters.lattice_vector
                    ) {
                        *pos_new = Some(pos);
                        *new_c = -c;
                    }
                    changed_lattice_vector = true;
                }

                if !new_pos.contains(&None) {
                    let new_pos = new_pos.map(Option::unwrap);
                    let new_index = self.index(&new_pos);
                    let new_i = match changed_lattice_vector {
                        true => self
                            .lattice_parameters
                            .iter()
                            .position(|lattice_parameters| {
                                lattice_parameters.lattice_vector == new_lattice_vector
                            })
                            .unwrap(),
                        false => i,
                    };
                    self.algorithm_values[new_index].distributions[new_i] =
                        self.algorithm_values[index].collision_distributions[i];
                }
            }

            if !self.next_pos(&mut pos, &dims) {
                break;
            }
        }
    }

    /// Calculate derived values.
    fn calculate_derived(&mut self) {
        let (mut pos, dims) = ([0; N], [true; N]);
        loop {
            let index = self.index(&pos);

            if self.object[index] {
                match self.next_pos(&mut pos, &dims) {
                    true => continue,
                    false => break,
                }
            }

            let algorithm_values = &mut self.algorithm_values[index];

            // calculate density
            algorithm_values.density = algorithm_values.distributions.iter().sum();

            // calculate velocity vector
            algorithm_values.velocity_vector.fill(0.0);
            if algorithm_values.density > 0.0 {
                for (lattice_parameters, f) in
                    izip!(&self.lattice_parameters, algorithm_values.distributions)
                {
                    for (u, c) in izip!(
                        &mut algorithm_values.velocity_vector,
                        lattice_parameters.lattice_vector
                    ) {
                        *u += c as f32 * f;
                    }
                }
                for u in &mut algorithm_values.velocity_vector {
                    *u /= algorithm_values.density;
                }
            }

            if !self.next_pos(&mut pos, &dims) {
                break;
            }
        }
    }

    /// Update inflows and the outflows.
    fn update_inflows_and_outflows(&mut self) {
        for (i, boundary_schemes) in self.boundary_schemes.iter().enumerate() {
            match boundary_schemes[0] {
                BoundaryScheme::Inflow => {
                    let (mut pos, mut dims) = ([0; N], [true; N]);
                    dims[i] = false;
                    loop {
                        let index = self.index(&pos);
                        self.algorithm_values[index] = self.source_algorithm_values;

                        if !self.next_pos(&mut pos, &dims) {
                            break;
                        }
                    }
                }
                BoundaryScheme::Outflow => {
                    let (mut pos, mut dims) = ([0; N], [true; N]);
                    dims[i] = false;
                    loop {
                        let mut other_pos = pos;
                        other_pos[i] += 1;

                        let (index, other_index) = (self.index(&pos), self.index(&other_pos));
                        self.algorithm_values[index] = self.algorithm_values[other_index];

                        if !self.next_pos(&mut pos, &dims) {
                            break;
                        }
                    }
                }
                _ => {}
            }
            match boundary_schemes[1] {
                BoundaryScheme::Inflow => {
                    let (mut pos, mut dims) = ([0; N], [true; N]);
                    (pos[i], dims[i]) = (self.size[i] - 1, false);
                    loop {
                        let index = self.index(&pos);
                        self.algorithm_values[index] = self.source_algorithm_values;

                        if !self.next_pos(&mut pos, &dims) {
                            break;
                        }
                    }
                }
                BoundaryScheme::Outflow => {
                    let (mut pos, mut dims) = ([0; N], [true; N]);
                    (pos[i], dims[i]) = (self.size[i] - 1, false);
                    loop {
                        let mut other_pos = pos;
                        other_pos[i] -= 1;

                        let (index, other_index) = (self.index(&pos), self.index(&other_pos));
                        self.algorithm_values[index] = self.algorithm_values[other_index];

                        if !self.next_pos(&mut pos, &dims) {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Calculate equilibrium distributions.
    fn equilibrium_distributions(
        lattice_parameters: &[LatticeParameters<N>; B],
        sound_speed_squared: f32,
        density: f32,
        velocity_vector: &[f32; N],
    ) -> [f32; B] {
        let cs2x2 = sound_speed_squared + sound_speed_squared;
        let cs4x2 = {
            let cs4 = sound_speed_squared * sound_speed_squared;
            cs4 + cs4
        };
        let u_dot_u = velocity_vector.dot_product(velocity_vector);

        let mut result = [0.0; B];
        for (val, lattice_parameters) in izip!(&mut result, lattice_parameters) {
            let c_dot_u = lattice_parameters
                .lattice_vector
                .map(|val| val as f32)
                .dot_product(velocity_vector);
            *val = lattice_parameters.weight
                * density
                * (1.0 + c_dot_u / sound_speed_squared + (c_dot_u * c_dot_u) / cs4x2
                    - u_dot_u / cs2x2);
        }
        result
    }
}
