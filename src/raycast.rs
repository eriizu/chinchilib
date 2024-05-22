use std::usize;

use array2d::Array2D;

pub struct World {
    map: Array2D<u8>,
    player_pos: (f32, f32),
    player_heading: f32,
    player_fov: f32,
}

impl Default for World {
    fn default() -> Self {
        let mut map = Array2D::filled_with(' ' as u8, 5, 5);
        map[(0, 0)] = 'X' as u8;
        map[(0, 1)] = 'X' as u8;
        map[(0, 2)] = 'X' as u8;
        map[(0, 3)] = 'X' as u8;
        map[(0, 4)] = 'X' as u8;
        map[(1, 0)] = 'X' as u8;
        map[(2, 0)] = 'X' as u8;
        map[(3, 0)] = 'X' as u8;
        map[(4, 0)] = 'X' as u8;
        map[(4, 0)] = 'X' as u8;
        map[(4, 1)] = 'X' as u8;
        map[(4, 2)] = 'X' as u8;
        map[(4, 3)] = 'X' as u8;
        map[(4, 4)] = 'X' as u8;
        map[(1, 4)] = 'X' as u8;
        map[(2, 4)] = 'X' as u8;
        map[(3, 4)] = 'X' as u8;
        map[(4, 4)] = 'X' as u8;
        Self {
            map,
            player_pos: (2.0, 2.0),
            player_heading: 0.0,
            player_fov: degs_to_rads(70),
        }
    }
}

pub enum Heading {
    Forward,
    Backward,
    Right,
    Left,
}

impl World {
    fn is_wall(&self, coords: (f32, f32)) -> bool {
        let coords = (coords.0 as usize, coords.1 as usize);
        coords.0 >= self.map.row_len()
            || coords.1 >= self.map.column_len()
            || self.map[coords] == 'X' as u8
    }

    fn distance_to_wall(&self, heading: f32) -> f32 {
        let mut distance: f32 = 0.0;
        let mut coords = move_forward(self.player_pos, heading, distance);
        while !self.is_wall(coords) {
            distance += 0.01;
            coords = move_forward(self.player_pos, heading, distance);
        }
        return distance;
    }

    pub fn distance_to_walls<'a>(&'a self, ray_quantity: usize) -> impl Iterator<Item = f32> + 'a {
        generate_ray_angles(ray_quantity, self.player_fov)
            .map(|angle| self.distance_to_wall(angle + self.player_heading))
    }

    pub fn pan_left(&mut self) {
        self.player_heading -= std::f32::consts::FRAC_PI_8;
        log::debug!("heading {}", rads_to_deg(self.player_heading));
    }
    pub fn pan_right(&mut self) {
        self.player_heading += std::f32::consts::FRAC_PI_8;
        log::debug!("heading {}", rads_to_deg(self.player_heading));
    }

    pub fn move_player(&mut self, heading: Heading) {
        let patate = self.player_heading
            + match heading {
                Heading::Forward => 0.0,
                Heading::Backward => std::f32::consts::PI,
                Heading::Left => 0.0 - std::f32::consts::FRAC_PI_2,
                Heading::Right => std::f32::consts::FRAC_PI_2,
            };
        let new_pos = move_forward(self.player_pos, patate, 0.2);
        if !self.is_wall(new_pos) {
            self.player_pos = new_pos;
        }
    }
}

pub fn move_forward(pos: (f32, f32), direction: f32, distance: f32) -> (f32, f32) {
    let x = pos.0 + direction.cos() * distance;
    let y = pos.1 + direction.sin() * distance;
    return (x, y);
}

pub fn move_forward_floored(pos: (usize, usize), direction: f32, distance: f32) -> (usize, usize) {
    let (x, y) = move_forward((pos.0 as f32, pos.1 as f32), direction, distance);
    (x as usize, y as usize)
}

fn generate_ray_angles(ray_quantity: usize, fov: f32) -> impl Iterator<Item = f32> {
    let lower_half = (fov / 2.0) * -1.0;
    let step = fov / (ray_quantity - 1) as f32;
    (0..ray_quantity)
        .enumerate()
        .map(move |(idx, _)| (step * idx as f32) + lower_half)
}

fn degs_to_rads(degs: u32) -> f32 {
    degs as f32 * (std::f32::consts::PI / 180.0)
}

#[allow(dead_code)]
fn rads_to_deg(rads: f32) -> u32 {
    (rads * (180.0 / std::f32::consts::PI)) as u32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn move_forward_floored_1() {
        let res = move_forward_floored((0, 0), 0.0, 1.0);
        assert_eq!(res, (1, 0));
        let res = move_forward_floored((0, 0), 0.0, 2.0);
        assert_eq!(res, (2, 0));
        let res = move_forward_floored((10, 20), 0.0, 2.0);
        assert_eq!(res, (12, 20));
    }

    #[test]
    fn move_forward_1() {
        let res = move_forward((0.0, 0.0), 0.0, 1.0);
        assert_eq!(res, (1.0, 0.0));
        let res = move_forward((0.0, 0.0), std::f32::consts::PI, 1.0);
        assert!(res.0 < -0.999);
        assert!(res.0 > -1.009);
        assert!(res.1 > -0.001);
        assert!(res.1 < 0.001);
        let res = move_forward((0.0, 0.0), std::f32::consts::FRAC_PI_2, 1.0);
        // assert_eq!(res, (0.0, 1.0));
        assert!(res.0 > -0.001);
        assert!(res.0 < 0.001);
        assert!(res.1 > 0.999);
        assert!(res.1 < 1.009);
        let res = move_forward((0.0, 0.0), std::f32::consts::FRAC_PI_2 * 3.0, 1.0);
        assert!(res.0 > -0.001);
        assert!(res.0 < 0.001);
        assert!(res.1 < -0.999);
        assert!(res.1 > -1.009);
    }

    #[test]
    fn generate_ray_angles_odd_1() {
        let res: Vec<f32> = generate_ray_angles(3, 3.0).collect();
        dbg!(&res);
        assert!(res[0] > -1.6);
        assert!(res[0] < 1.4);
        assert!(res[1] > -0.01);
        assert!(res[1] < 0.01);
        assert!(res[2] < 1.6);
        assert!(res[2] > 1.4);
    }
    #[test]
    fn generate_ray_angles_odd_2() {
        let res: Vec<f32> = generate_ray_angles(5, 4.0).collect();
        dbg!(&res);
        assert!(res[0] > -2.01);
        assert!(res[0] < 1.99);
        assert!(res[1] > -1.01);
        assert!(res[1] < 0.99);
        assert!(res[2] > -0.01);
        assert!(res[2] < 0.01);
        assert!(res[3] > 0.99);
        assert!(res[3] < 1.01);
        assert!(res[4] > 1.99);
        assert!(res[4] < 2.01);
    }
    #[test]
    fn generate_ray_angles_even() {
        let res: Vec<f32> = generate_ray_angles(4, 4.0).collect();
        dbg!(&res);
        assert!(res[0] > -2.01);
        assert!(res[0] < 1.99);
        assert!(res[1] > -0.67);
        assert!(res[1] < -0.66);
        assert!(res[2] > 0.66);
        assert!(res[2] < 0.67);
        assert!(res[3] > 1.99);
        assert!(res[3] < 2.01);
    }
}
