use num_bigint::{BigUint, RandBigInt};
use rand;

pub struct chaum_pedersen {
    p: BigUint,
    q: BigUint,
    alpha: BigUint,
    beta: BigUint,
}

impl chaum_pedersen {
    /// alpha^x mod p
    /// output = n^exp mod p
    pub fn exponentiate(n: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
        n.modpow(exponent, modulus)
    }

    /// output = s = k - c * x mod q
    pub fn solve(&self, k: &BigUint, c: &BigUint, x: &BigUint) -> BigUint {
        if *k >= c * x {
            return (k - c * x).modpow(&BigUint::from(1u32), &self.q);
        }
        return &self.q - (c * x - k).modpow(&BigUint::from(1u32), &self.q);
    }

    /// r1 = alpha^s * y1^c
    /// r2 = beta^s * y2^c
    pub fn verify(
        &self,
        y1: &BigUint,
        y2: &BigUint,
        r1: &BigUint,
        r2: &BigUint,
        c: &BigUint,
        s: &BigUint,
    ) -> bool {
        let condition1 = *r1
            == (&self.alpha.modpow(s, &self.p) * y1.modpow(c, &self.p))
                .modpow(&BigUint::from(1u32), &self.p);
        let condition2 = *r2
            == (&self.beta.modpow(s, &self.p) * y2.modpow(c, &self.p))
                .modpow(&BigUint::from(1u32), &self.p);

        println!("r1 -> {}", r1);
        println!("r2 -> {}", r2);
        println!("condition1 -> {}", condition1);
        println!("condition2 -> {}", condition2);

        condition1 && condition2
    }

    pub fn generate_random_below(bound: &BigUint) -> BigUint {
        let mut rng = rand::thread_rng();
        rng.gen_biguint_below(bound)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_trivial_example() {
        let alpha = BigUint::from(4u32);
        let beta = BigUint::from(9u32);
        let p = BigUint::from(23u32);
        let q = BigUint::from(11u32);
        let x = BigUint::from(6u32);
        let k = BigUint::from(7u32);
        let c = BigUint::from(4u32);

        let cp = chaum_pedersen {
            p: p.clone(),
            q,
            alpha: alpha.clone(),
            beta: beta.clone(),
        };

        let y1 = chaum_pedersen::exponentiate(&alpha, &x, &p);
        let y2 = chaum_pedersen::exponentiate(&beta, &x, &p);

        assert_eq!(y1, BigUint::from(2u32));
        assert_eq!(y2, BigUint::from(3u32));

        let r1 = chaum_pedersen::exponentiate(&alpha, &k, &p);
        let r2 = chaum_pedersen::exponentiate(&beta, &k, &p);

        assert_eq!(r1, BigUint::from(8u32));
        assert_eq!(r2, BigUint::from(4u32));

        let s = cp.solve(&k, &c, &x);
        assert_eq!(s, BigUint::from(5u32));

        let result = cp.verify(&y1, &y2, &r1, &r2, &c, &s);
        assert!(result);

        // fake secret
        // s = k - c * x mod p
        let x_fake = BigUint::from(7u32);
        let s_fake = cp.solve(&k, &c, &x_fake);

        let result = cp.verify(&y1, &y2, &r1, &r2, &c, &s_fake);
        assert!(!result);
    }

    #[test]
    fn test_trivial_example_with_random_numbers() {
        let alpha = BigUint::from(4u32);
        let beta = BigUint::from(9u32);
        let p = BigUint::from(23u32);
        let q = BigUint::from(11u32);
        let x = BigUint::from(6u32);
        let k = chaum_pedersen::generate_random_below(&q);
        let c = chaum_pedersen::generate_random_below(&q);

        let cp = chaum_pedersen {
            p: p.clone(),
            q,
            alpha: alpha.clone(),
            beta: beta.clone(),
        };

        let y1 = chaum_pedersen::exponentiate(&alpha, &x, &p);
        let y2 = chaum_pedersen::exponentiate(&beta, &x, &p);

        assert_eq!(y1, BigUint::from(2u32));
        assert_eq!(y2, BigUint::from(3u32));

        let r1 = chaum_pedersen::exponentiate(&alpha, &k, &p);
        let r2 = chaum_pedersen::exponentiate(&beta, &k, &p);

        let s = cp.solve(&k, &c, &x);

        let result = cp.verify(&y1, &y2, &r1, &r2, &c, &s);
        assert!(result);
    }
}
